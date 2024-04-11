#!/bin/bash
# Check if Zsh is installed, if not, install it.
if ! which zsh > /dev/null; then
    echo "Installing Zsh..."
    sudo apt install zsh
fi

# Check if zplug is installed, if not, install it.
if [ ! -d ~/.zplug ]; then
    echo "Installing zplug..."
    curl -sL --proto-redir -all,https https://raw.githubusercontent.com/zplug/installer/master/installer.zsh | zsh
fi

# Set Zsh as the default shell
sudo chsh -s $(which zsh) $(whoami)

# Check if Neovim is installed, if not, install it.
if ! which nvim > /dev/null; then
    echo "Installing Neovim..."
    curl -LO https://github.com/neovim/neovim/releases/latest/download/nvim.appimage
    chmod u+x nvim.appimage
    ./nvim.appimage --appimage-extract
    ./squashfs-root/AppRun --version
    # Optional: exposing nvim globally.
    sudo mv squashfs-root /
    sudo ln -s /squashfs-root/AppRun /usr/bin/nvim
    # Clone the NvChad repository.
    mv ~/.config/nvim ~/.config/nvim.bak
    rm -rf ~/.local/share/nvim
    git clone https://github.com/NvChad/NvChad ~/.config/nvim --depth 1
fi

# Create or overwrite the .zshrc file.
cat > ~/.zshrc <<EOF
# =====================
# Z-plug Plugin Manager
# =====================

# Source Z-plug
source ~/.zplug/init.zsh

# Plugins
zplug 'romkatv/powerlevel10k', as:theme, depth:1
zplug 'zsh-users/zsh-autosuggestions'
zplug 'zsh-users/zsh-history-substring-search'
zplug 'marlonrichert/zsh-autocomplete'
zplug 'hlissner/zsh-autopair'

# Check and install plugins if necessary
if ! zplug check --verbose; then
    printf "Install? [y/N]: "
    if read -q; then
        echo
        zplug install
    fi
fi

# Load plugins
zplug load

# =====================
# Keybindings and History
# =====================

# Keybindings for history substring search
bindkey "\$terminfo[kcuu1]" history-substring-search-up
bindkey "\$terminfo[kcud1]" history-substring-search-down

# History settings
SAVEHIST=1000
export HISTFILE=~/.zsh_history
setopt share_history

# =====================
# Custom Aliases and Functions
# =====================

alias rm='rm -r'
alias cp='cp -r'
alias ls='ls -hlF --color=auto'
alias ..='cd ../'
alias tree="tree -alI 'node_modules|.git'"
alias grep='grep --color=always'
alias grepFind='grep --exclude-dir=node_modules -nr . -e'
alias mkdir='mkdir -p'
alias vim='nvim'

# =====================
# Powerlevel10k Customization
# =====================

# Load Powerlevel10k instant prompt
if [[ -r "\${XDG_CACHE_HOME:-\$HOME/.cache}/p10k-instant-prompt-\${(%):-%n}.zsh" ]]; then
  source "\${XDG_CACHE_HOME:-\$HOME/.cache}/p10k-instant-prompt-\${(%):-%n}.zsh"
fi

# To customize the prompt, run \`p10k configure\` or edit ~/.p10k.zsh
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh

# =====================
# Additional Customizations
# =====================

# Add your additional customizations below this section

# Example: Export environment variables
# export MY_VARIABLE="example_value"

EOF

# Inform the user
echo "Zsh configuration and dependencies installed. Please restart your shell."
