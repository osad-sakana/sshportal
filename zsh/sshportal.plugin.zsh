#!/bin/zsh

# sshportal zsh plugin

# Completion function for sshportal
_sshportal() {
    local context state line
    typeset -A opt_args

    _arguments \
        '1: :_sshportal_commands' \
        '*:: :->args' && return 0

    case $state in
        args)
            case $words[1] in
                add-host|add-paths)
                    # No arguments needed for interactive commands
                    ;;
                remove-host|connect)
                    _arguments '1:host:_sshportal_hosts'
                    ;;
                add-path)
                    _arguments \
                        '1:name:' \
                        '2:path:_files' \
                        '(-r --remote)'{-r,--remote}'[Mark as remote path]'
                    ;;
                remove-path)
                    _arguments '1:path:_sshportal_paths'
                    ;;
                copy)
                    _arguments \
                        '1:source:_sshportal_copy_source' \
                        '2:destination:_sshportal_copy_destination'
                    ;;
            esac
            ;;
    esac
}

# Get available commands
_sshportal_commands() {
    local commands
    commands=(
        'add-host:Add a new host (interactive)'
        'remove-host:Remove a host'
        'list-hosts:List all configured hosts'
        'connect:Connect to a host'
        'add-path:Add a path alias (legacy)'
        'remove-path:Remove a path alias'
        'list-paths:List all configured paths'
        'copy:Copy files using SCP with path aliases'
        'add-paths:Add path aliases (interactive)'
    )
    _describe 'commands' commands
}

# Get configured hosts
_sshportal_hosts() {
    local hosts
    if command -v sshportal >/dev/null 2>&1; then
        hosts=(${(f)"$(sshportal list-hosts 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
        _describe 'hosts' hosts
    fi
}

# Get configured paths
_sshportal_paths() {
    local paths
    if command -v sshportal >/dev/null 2>&1; then
        paths=(${(f)"$(sshportal list-paths 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
        _describe 'paths' paths
    fi
}

# Get sources for copy command (paths and files)
_sshportal_copy_source() {
    local -a sources
    if command -v sshportal >/dev/null 2>&1; then
        # Get local paths
        local local_paths=(${(f)"$(sshportal list-paths 2>/dev/null | grep '(local)' | awk '{print $1}' | sort)"})
        sources+=($local_paths)
    fi
    # Add file completion
    _alternative \
        'paths:paths:compadd -a sources' \
        'files:files:_files'
}

# Get destinations for copy command (hosts:paths and local paths)
_sshportal_copy_destination() {
    local -a destinations
    if command -v sshportal >/dev/null 2>&1; then
        # Get hosts for remote destinations
        local hosts=(${(f)"$(sshportal list-hosts 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
        local remote_paths=(${(f)"$(sshportal list-paths 2>/dev/null | grep '(remote)' | awk '{print $1}' | sort)"})
        local local_paths=(${(f)"$(sshportal list-paths 2>/dev/null | grep '(local)' | awk '{print $1}' | sort)"})
        
        # Create host:path combinations
        for host in $hosts; do
            for path in $remote_paths; do
                destinations+=("$host:$path")
            done
        done
        
        destinations+=($local_paths)
    fi
    
    _alternative \
        'destinations:destinations:compadd -a destinations' \
        'files:files:_files'
}

# Enhanced ssh completion with hosts
_ssh_hosts_enhanced() {
    local -a ssh_hosts sshportal_hosts
    
    # Get default ssh hosts
    ssh_hosts=(${(f)"$(awk '/^Host [^*]/ {print $2}' ~/.ssh/config 2>/dev/null)"})
    
    # Get sshportal hosts
    if command -v sshportal >/dev/null 2>&1; then
        sshportal_hosts=(${(f)"$(sshportal list-hosts 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
    fi
    
    _alternative \
        'ssh-hosts:ssh hosts:compadd -a ssh_hosts' \
        'sshportal-hosts:sshportal hosts:compadd -a sshportal_hosts'
}

# Enhanced scp completion with paths and hosts
_scp_enhanced() {
    if [[ $CURRENT -eq 2 || $CURRENT -eq 3 ]]; then
        local -a sources destinations
        
        if command -v sshportal >/dev/null 2>&1; then
            local hosts=(${(f)"$(sshportal list-hosts 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
            local paths=(${(f)"$(sshportal list-paths 2>/dev/null | grep '^  ' | awk '{print $1}' | sort)"})
            
            # Add host: prefixes for remote paths
            for host in $hosts; do
                for path in $paths; do
                    sources+=("$host:$path")
                    destinations+=("$host:$path")
                done
            done
            
            sources+=($paths)
            destinations+=($paths)
        fi
        
        _alternative \
            'sshportal-paths:sshportal paths:compadd -a sources' \
            'files:files:_files'
    else
        _files
    fi
}

# Register completions
compdef _sshportal sshportal
compdef _ssh_hosts_enhanced ssh
compdef _scp_enhanced scp

# Aliases for convenience
alias sp='sshportal'
alias spc='sshportal connect'
alias spl='sshportal list-hosts'
alias spp='sshportal list-paths'