_bashers_complete() {
  local cur
  cur="${COMP_WORDS[COMP_CWORD]}"
  local commands
  commands="$(bashers _commands 2>/dev/null)"
  COMPREPLY=($(compgen -W "${commands}" -- "$cur"))
  return 0
}

complete -F _bashers_complete bashers
