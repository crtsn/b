#!/bin/bash

npm uninstall -g @anthropic-ai/claude-code 1>/dev/null 2>&1
rm -f ~/.local/bin/claude 1>/dev/null 2>&1
rm -rf ~/.local/share/claude 1>/dev/null 2>&1
claude uninstall 1>/dev/null 2>&1
ls $@
