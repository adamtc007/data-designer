#!/bin/bash
# Cron job setup script for Elasticsearch log autopurge

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOPURGE_SCRIPT="$SCRIPT_DIR/elasticsearch-autopurge.sh"
CRON_LOG_FILE="/var/log/elasticsearch-autopurge-cron.log"

# Function to set up cron job
setup_cron() {
    echo "Setting up Elasticsearch log autopurge cron job..."

    # Check if cron job already exists
    if crontab -l 2>/dev/null | grep -q "elasticsearch-autopurge.sh"; then
        echo "Cron job already exists. Current crontab:"
        crontab -l | grep "elasticsearch-autopurge.sh"
        echo ""
        read -p "Do you want to replace it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Keeping existing cron job."
            return 0
        fi

        # Remove existing cron job
        crontab -l | grep -v "elasticsearch-autopurge.sh" | crontab -
    fi

    echo "Available schedule options:"
    echo "1. Daily at 2 AM"
    echo "2. Daily at 6 AM"
    echo "3. Weekly on Sunday at 2 AM"
    echo "4. Custom schedule"
    echo ""

    read -p "Select schedule option (1-4): " schedule_option

    case $schedule_option in
        1)
            cron_schedule="0 2 * * *"
            ;;
        2)
            cron_schedule="0 6 * * *"
            ;;
        3)
            cron_schedule="0 2 * * 0"
            ;;
        4)
            echo "Enter custom cron schedule (e.g., '0 2 * * *' for daily at 2 AM):"
            read -p "Schedule: " cron_schedule
            ;;
        *)
            echo "Invalid option. Using default: daily at 2 AM"
            cron_schedule="0 2 * * *"
            ;;
    esac

    # Add new cron job
    (crontab -l 2>/dev/null; echo "$cron_schedule $AUTOPURGE_SCRIPT >> $CRON_LOG_FILE 2>&1") | crontab -

    echo "✅ Cron job added successfully!"
    echo "Schedule: $cron_schedule"
    echo "Script: $AUTOPURGE_SCRIPT"
    echo "Log file: $CRON_LOG_FILE"
    echo ""
    echo "Current crontab:"
    crontab -l
}

# Function to remove cron job
remove_cron() {
    echo "Removing Elasticsearch log autopurge cron job..."

    if ! crontab -l 2>/dev/null | grep -q "elasticsearch-autopurge.sh"; then
        echo "No cron job found for elasticsearch-autopurge.sh"
        return 0
    fi

    # Remove cron job
    crontab -l | grep -v "elasticsearch-autopurge.sh" | crontab -

    echo "✅ Cron job removed successfully!"
}

# Function to show current status
show_status() {
    echo "Elasticsearch Log Autopurge Status:"
    echo "=================================="

    if crontab -l 2>/dev/null | grep -q "elasticsearch-autopurge.sh"; then
        echo "✅ Cron job is active:"
        crontab -l | grep "elasticsearch-autopurge.sh"
    else
        echo "❌ No cron job found"
    fi

    echo ""
    echo "Script location: $AUTOPURGE_SCRIPT"
    echo "Log file: $CRON_LOG_FILE"

    if [ -f "$CRON_LOG_FILE" ]; then
        echo ""
        echo "Recent log entries:"
        tail -10 "$CRON_LOG_FILE"
    fi

    echo ""
    echo "Test Elasticsearch connection:"
    if curl -s "http://localhost:9200/_cluster/health" > /dev/null; then
        echo "✅ Elasticsearch is available"

        # Show current test indices
        echo ""
        echo "Current test log indices:"
        curl -s "http://localhost:9200/_cat/indices/test-logs-*?v" || echo "No test-logs indices found"
    else
        echo "❌ Elasticsearch is not available"
    fi
}

# Function to test the autopurge script
test_autopurge() {
    echo "Testing Elasticsearch autopurge script (dry run)..."
    echo "=================================================="

    if [ ! -f "$AUTOPURGE_SCRIPT" ]; then
        echo "❌ Autopurge script not found: $AUTOPURGE_SCRIPT"
        exit 1
    fi

    # Run dry run
    "$AUTOPURGE_SCRIPT" --dry-run
}

# Show help
show_help() {
    cat << EOF
Elasticsearch Log Autopurge Cron Setup

USAGE:
    $0 [COMMAND]

COMMANDS:
    setup       Set up cron job for automatic log cleanup
    remove      Remove existing cron job
    status      Show current status and recent logs
    test        Test autopurge script (dry run)
    help        Show this help message

EXAMPLES:
    # Set up daily cleanup at 2 AM
    $0 setup

    # Check current status
    $0 status

    # Test the cleanup script
    $0 test

    # Remove automatic cleanup
    $0 remove

The autopurge script will:
- Delete test log indices older than 7 days
- Keep a log of all cleanup operations
- Optimize remaining indices for better performance
- Generate reports of current log storage usage
EOF
}

# Main script logic
case "${1:-help}" in
    setup)
        setup_cron
        ;;
    remove)
        remove_cron
        ;;
    status)
        show_status
        ;;
    test)
        test_autopurge
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac