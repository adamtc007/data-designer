#!/bin/bash
# Elasticsearch management script for Data Designer

set -e

function check_docker() {
    echo "🐳 Checking Docker status..."

    # Check if Docker daemon is running
    if ! docker info >/dev/null 2>&1; then
        echo "⚠️  Docker daemon not running. Starting Docker Desktop..."

        # Start Docker Desktop on macOS
        if [[ "$OSTYPE" == "darwin"* ]]; then
            open /Applications/Docker.app
            echo "⏳ Waiting for Docker Desktop to start..."

            # Wait up to 60 seconds for Docker to be ready
            local count=0
            while ! docker info >/dev/null 2>&1 && [ $count -lt 60 ]; do
                sleep 2
                count=$((count + 2))
                echo -n "."
            done
            echo ""

            if docker info >/dev/null 2>&1; then
                echo "✅ Docker Desktop started successfully"
            else
                echo "❌ Failed to start Docker Desktop. Please start it manually."
                exit 1
            fi
        else
            echo "❌ Docker daemon not running. Please start Docker manually."
            exit 1
        fi
    else
        echo "✅ Docker daemon is running"
    fi
}

function show_help() {
    echo "🔍 Elasticsearch Management for Data Designer"
    echo "=============================================="
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  start    - Start Elasticsearch and Kibana containers"
    echo "  stop     - Stop Elasticsearch and Kibana containers"
    echo "  restart  - Restart Elasticsearch and Kibana containers"
    echo "  status   - Show container status and health"
    echo "  logs     - Show container logs"
    echo "  health   - Check Elasticsearch cluster health"
    echo "  clean    - Stop containers and remove volumes (⚠️  DESTRUCTIVE)"
    echo "  help     - Show this help message"
    echo ""
    echo "URLs:"
    echo "  Elasticsearch: http://localhost:9200"
    echo "  Kibana:        http://localhost:5601"
}

function start_services() {
    check_docker
    echo "🚀 Starting Elasticsearch and Kibana..."
    docker compose up -d
    echo "⏳ Waiting for services to start..."
    sleep 10
    show_status
}

function stop_services() {
    echo "🛑 Stopping Elasticsearch and Kibana..."
    docker compose down
}

function restart_services() {
    check_docker
    echo "🔄 Restarting Elasticsearch and Kibana..."
    stop_services
    sleep 2
    start_services
}

function show_status() {
    echo "📊 Container Status:"
    docker compose ps
    echo ""

    echo "🏥 Health Check:"
    if curl -s http://localhost:9200/_cluster/health >/dev/null 2>&1; then
        echo "✅ Elasticsearch: Running"
        curl -s http://localhost:9200/_cluster/health | jq -r '"   Status: \(.status) | Nodes: \(.number_of_nodes) | Shards: \(.active_shards)"'
    else
        echo "❌ Elasticsearch: Not accessible"
    fi

    if curl -s http://localhost:5601/api/status >/dev/null 2>&1; then
        echo "✅ Kibana: Running"
    else
        echo "⏳ Kibana: Starting or not accessible"
    fi
}

function show_logs() {
    echo "📋 Recent logs (last 20 lines per service):"
    echo ""
    echo "=== Elasticsearch Logs ==="
    docker compose logs --tail=20 elasticsearch
    echo ""
    echo "=== Kibana Logs ==="
    docker compose logs --tail=20 kibana
}

function check_health() {
    echo "🏥 Elasticsearch Cluster Health:"
    if curl -s http://localhost:9200/_cluster/health; then
        echo ""
        echo ""
        echo "📊 Cluster Stats:"
        curl -s http://localhost:9200/_cluster/stats | jq '{
            cluster_name: .cluster_name,
            status: .status,
            indices: .indices.count,
            docs: .indices.docs.count,
            store_size: .indices.store.size_in_bytes
        }'
    else
        echo "❌ Elasticsearch not accessible at http://localhost:9200"
        exit 1
    fi
}

function clean_all() {
    echo "⚠️  This will stop containers and remove all data volumes!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🧹 Stopping containers and removing volumes..."
        docker compose down -v
        echo "✅ Cleanup complete. All data has been removed."
    else
        echo "❌ Cleanup cancelled."
    fi
}

# Main script logic
case "${1:-help}" in
    start)
        start_services
        ;;
    stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    health)
        check_health
        ;;
    clean)
        clean_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac