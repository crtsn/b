#!/bin/bash

npm uninstall -g @anthropic-ai/claude-code
rm -f ~/.local/bin/claude
rm -rf ~/.local/share/claude
claude uninstall
ls $@
