#!/bin/bash
# Elasticsearch Log Autopurge Script
# Automatically removes test logs older than 7 days

set -e

# Configuration
ELASTICSEARCH_URL=${ELASTICSEARCH_URL:-"http://localhost:9200"}
RETENTION_DAYS=${RETENTION_DAYS:-7}
LOG_FILE="/var/log/elasticsearch-autopurge.log"
DRY_RUN=${DRY_RUN:-false}

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Check if Elasticsearch is available
check_elasticsearch() {
    log "🔍 Checking Elasticsearch connection..."

    if ! curl -s "$ELASTICSEARCH_URL/_cluster/health" > /dev/null; then
        log "❌ Elasticsearch is not available at $ELASTICSEARCH_URL"
        exit 1
    fi

    log "✅ Elasticsearch is available"
}

# Get indices older than retention period
get_old_indices() {
    local cutoff_date=$(date -d "$RETENTION_DAYS days ago" +%Y-%m-%d)

    log "📅 Finding indices older than $cutoff_date (retention: $RETENTION_DAYS days)"

    # Get all test-logs indices with their creation dates
    curl -s "$ELASTICSEARCH_URL/_cat/indices/test-logs-*?h=index,creation.date.string&format=json" | \
    jq -r --arg cutoff "$cutoff_date" '
        .[] |
        select(.["creation.date.string"] < $cutoff) |
        .index
    '
}

# Delete old indices
delete_old_indices() {
    local indices_to_delete=($(get_old_indices))

    if [ ${#indices_to_delete[@]} -eq 0 ]; then
        log "✅ No indices found older than $RETENTION_DAYS days"
        return 0
    fi

    log "📋 Found ${#indices_to_delete[@]} indices to delete:"

    for index in "${indices_to_delete[@]}"; do
        log "   - $index"
    done

    if [ "$DRY_RUN" = "true" ]; then
        log "🏃 DRY RUN: Would delete ${#indices_to_delete[@]} indices"
        return 0
    fi

    # Delete indices one by one
    local deleted_count=0
    local failed_count=0

    for index in "${indices_to_delete[@]}"; do
        log "🗑️  Deleting index: $index"

        if curl -s -X DELETE "$ELASTICSEARCH_URL/$index" | jq -r '.acknowledged' | grep -q "true"; then
            log "✅ Successfully deleted: $index"
            ((deleted_count++))
        else
            log "❌ Failed to delete: $index"
            ((failed_count++))
        fi
    done

    log "📊 Deletion summary: $deleted_count deleted, $failed_count failed"

    if [ $failed_count -gt 0 ]; then
        exit 1
    fi
}

# Clean up old documents within indices (alternative to deleting entire indices)
cleanup_old_documents() {
    local cutoff_timestamp=$(date -d "$RETENTION_DAYS days ago" --iso-8601=seconds)

    log "📄 Cleaning up documents older than $cutoff_timestamp"

    # Delete documents older than retention period
    local delete_query='{
        "query": {
            "range": {
                "timestamp": {
                    "lt": "'$cutoff_timestamp'"
                }
            }
        }
    }'

    if [ "$DRY_RUN" = "true" ]; then
        # Count documents that would be deleted
        local count_query='{
            "query": {
                "range": {
                    "timestamp": {
                        "lt": "'$cutoff_timestamp'"
                    }
                }
            }
        }'

        local doc_count=$(curl -s -X GET "$ELASTICSEARCH_URL/test-logs-*/_count" \
            -H "Content-Type: application/json" \
            -d "$count_query" | jq -r '.count')

        log "🏃 DRY RUN: Would delete $doc_count documents"
        return 0
    fi

    local response=$(curl -s -X POST "$ELASTICSEARCH_URL/test-logs-*/_delete_by_query?conflicts=proceed" \
        -H "Content-Type: application/json" \
        -d "$delete_query")

    local deleted=$(echo "$response" | jq -r '.deleted')
    local failures=$(echo "$response" | jq -r '.failures | length')

    log "📊 Document cleanup: $deleted deleted, $failures failures"

    if [ "$failures" != "0" ]; then
        log "⚠️  Some document deletions failed"
        echo "$response" | jq '.failures' | tee -a "$LOG_FILE"
    fi
}

# Optimize indices after cleanup
optimize_indices() {
    log "⚡ Optimizing remaining indices..."

    if [ "$DRY_RUN" = "true" ]; then
        log "🏃 DRY RUN: Would optimize indices"
        return 0
    fi

    # Force merge to reclaim space
    curl -s -X POST "$ELASTICSEARCH_URL/test-logs-*/_forcemerge?max_num_segments=1" > /dev/null

    log "✅ Index optimization completed"
}

# Generate cleanup report
generate_report() {
    log "📋 Generating cleanup report..."

    local total_indices=$(curl -s "$ELASTICSEARCH_URL/_cat/indices/test-logs-*?h=index" | wc -l)
    local total_docs=$(curl -s "$ELASTICSEARCH_URL/test-logs-*/_count" | jq -r '.count')
    local total_size=$(curl -s "$ELASTICSEARCH_URL/_cat/indices/test-logs-*?h=store.size&bytes=b" | \
        awk '{sum += $1} END {printf "%.2f MB", sum/1024/1024}')

    log "📊 Current state:"
    log "   - Total indices: $total_indices"
    log "   - Total documents: $total_docs"
    log "   - Total size: $total_size"

    # Get oldest and newest log dates
    local oldest=$(curl -s "$ELASTICSEARCH_URL/test-logs-*/_search" \
        -H "Content-Type: application/json" \
        -d '{"size":1,"sort":[{"timestamp":{"order":"asc"}}],"_source":["timestamp"]}' | \
        jq -r '.hits.hits[0]._source.timestamp // "N/A"')

    local newest=$(curl -s "$ELASTICSEARCH_URL/test-logs-*/_search" \
        -H "Content-Type: application/json" \
        -d '{"size":1,"sort":[{"timestamp":{"order":"desc"}}],"_source":["timestamp"]}' | \
        jq -r '.hits.hits[0]._source.timestamp // "N/A"')

    log "   - Oldest log: $oldest"
    log "   - Newest log: $newest"
}

# Show help
show_help() {
    cat << EOF
Elasticsearch Test Logs Autopurge

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --dry-run           Show what would be deleted without actually deleting
    --retention-days N  Set retention period in days (default: 7)
    --elasticsearch-url URL  Set Elasticsearch URL (default: http://localhost:9200)
    --documents-only    Only delete old documents, keep indices
    --help              Show this help message

EXAMPLES:
    # Dry run to see what would be deleted
    $0 --dry-run

    # Delete indices older than 14 days
    $0 --retention-days 14

    # Delete documents only (keep index structure)
    $0 --documents-only

ENVIRONMENT VARIABLES:
    ELASTICSEARCH_URL   Elasticsearch endpoint
    RETENTION_DAYS      Number of days to retain logs
    DRY_RUN            Set to 'true' for dry run mode

CRON SETUP:
    # Run daily at 2 AM
    0 2 * * * /path/to/elasticsearch-autopurge.sh >> /var/log/elasticsearch-autopurge.log 2>&1
EOF
}

# Parse command line arguments
DOCUMENTS_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --retention-days)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --elasticsearch-url)
            ELASTICSEARCH_URL="$2"
            shift 2
            ;;
        --documents-only)
            DOCUMENTS_ONLY=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log "🚀 Starting Elasticsearch test logs autopurge"
    log "⚙️  Configuration:"
    log "   - Elasticsearch URL: $ELASTICSEARCH_URL"
    log "   - Retention days: $RETENTION_DAYS"
    log "   - Dry run: $DRY_RUN"
    log "   - Documents only: $DOCUMENTS_ONLY"

    check_elasticsearch

    if [ "$DOCUMENTS_ONLY" = "true" ]; then
        cleanup_old_documents
    else
        delete_old_indices
    fi

    if [ "$DRY_RUN" != "true" ]; then
        optimize_indices
    fi

    generate_report

    log "✅ Elasticsearch autopurge completed successfully"
}

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

# Run main function
main "$@"