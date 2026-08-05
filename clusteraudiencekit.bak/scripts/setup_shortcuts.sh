#!/bin/bash
# Setup keyboard shortcuts for ClusterAudienceKit

add_shortcuts() {
  if [ -f ~/.zshrc ]; then
    RC_FILE=~/.zshrc
  elif [ -f ~/.bashrc ]; then
    RC_FILE=~/.bashrc
  else
    echo "❌ No shell config found"; return 1
  fi
  
  if grep -q "dash-clusteraudiencekit" "$RC_FILE"; then
    echo "⚠️  Shortcuts already installed"; return 0
  fi
  
  cat >> "$RC_FILE" << 'ALIASES'

# ClusterAudienceKit dashboard shortcuts
alias dash-clusteraudiencekit='clusteraudiencekit dashboard --static'
alias dash-clusteraudiencekit-live='clusteraudiencekit dashboard'
alias dash-clusteraudiencekit-export='clusteraudiencekit dashboard --export /tmp/${pkg}_metrics.json && echo ✓ Exported'
ALIASES
  
  echo "✅ Shortcuts added to $RC_FILE"
  echo "   Run: source $RC_FILE"
}

remove_shortcuts() {
  sed -i '' '/# ClusterAudienceKit dashboard shortcuts/,/alias dash-clusteraudiencekit-export=/d' ~/.zshrc 2>/dev/null
  sed -i '' '/# ClusterAudienceKit dashboard shortcuts/,/alias dash-clusteraudiencekit-export=/d' ~/.bashrc 2>/dev/null
  echo "✅ Shortcuts removed"
}

case "${1:-}" in --remove) remove_shortcuts ;; *) add_shortcuts ;; esac
