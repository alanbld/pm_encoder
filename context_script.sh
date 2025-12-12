#!/bin/bash
# prepare_context.sh

echo "Preparing pm_encoder context with AI Collaboration Protocol..."

# Create temporary file
CONTEXT_FILE="/tmp/pm_encoder_full_context.txt"

# Add header
echo "# pm_encoder Development Context" > $CONTEXT_FILE
echo "# Generated: $(date)" >> $CONTEXT_FILE
echo "# Purpose: Generate SYSTEM_INSTRUCTIONS.md for pm_encoder" >> $CONTEXT_FILE
echo "" >> $CONTEXT_FILE

# Add the protocol first
echo "++++++++++ AI_COLLABORATION_PROTOCOL.md ++++++++++" >> $CONTEXT_FILE
cat ../ai_collaboration_protocol_system.md >> $CONTEXT_FILE
echo "---------- AI_COLLABORATION_PROTOCOL.md ----------" >> $CONTEXT_FILE
echo "" >> $CONTEXT_FILE

# Then add pm_encoder itself
./pm_encoder.py . --include "*.py" "*.md" "*.json" >> $CONTEXT_FILE

echo "Context prepared in: $CONTEXT_FILE"
echo "Size: $(wc -l $CONTEXT_FILE | cut -d' ' -f1) lines"
