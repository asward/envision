use crate::cli::Shell;

pub fn run(shell: &Shell) -> Result<u8, String> {
    let code = match shell {
        Shell::Bash => format!("{COMMON_HOOK}\n{BASH_PROMPT}"),
        Shell::Zsh => format!("{COMMON_HOOK}\n{ZSH_PROMPT}"),
        Shell::Fish => FISH_HOOK.to_string(),
    };
    print!("{code}");
    Ok(0)
}

/// Shared bash/zsh: envision wrapper + fixed-top banner using scroll regions.
const COMMON_HOOK: &str = r#"
envision() {
    case "$1" in
        session|set|unset|clear|profile)
            local _envision_out
            _envision_out="$(command envision "$@")"
            local _envision_rc=$?
            if [ $_envision_rc -eq 0 ] && [ -n "$_envision_out" ]; then
                eval "$_envision_out"
            fi
            return $_envision_rc
            ;;
        *)
            command envision "$@"
            ;;
    esac
}

_envision_banner() {
    [ "${ENVISION_BANNER}" = "off" ] && return
    [ -n "${TMUX}" ] && return
    case "${TERM}" in screen*|dumb) return ;; esac
    [ ! -t 2 ] && return

    # Nothing to show: tear down scroll region if it was active
    if [ -z "${ENVISION_SESSION}" ] && [ -z "${ENVISION_PROFILE}" ]; then
        if [ "${_ENVISION_BANNER_ACTIVE}" = "1" ]; then
            _ENVISION_BANNER_ACTIVE=0
            # Reset scroll region to full screen, clear banner line
            printf '\e[r\e7\e[1;1H\e[2K\e8' >&2
        fi
        return
    fi

    local _cols="${COLUMNS:-80}"
    local _lines="${LINES:-24}"
    local _parts=""

    if [ -n "${ENVISION_PROFILE}" ]; then
        _parts=" ${ENVISION_PROFILE}"
    fi

    if [ -n "${ENVISION_SESSION_ID}" ]; then
        local _state="clean"
        [ "${ENVISION_DIRTY}" = "1" ] && _state="dirty"
        local _tracked="${ENVISION_TRACKED:-0}"
        local _sess="${ENVISION_SESSION_ID} | ${_tracked} tracked | ${_state}"
        if [ -n "${_parts}" ]; then
            _parts="${_parts} | ${_sess}"
        else
            _parts=" ${_sess}"
        fi
    fi

    [ -z "${_parts}" ] && return
    _parts="${_parts} "

    local _len=${#_parts}
    local _pad=$(( _cols - _len ))
    [ $_pad -lt 0 ] && _pad=0

    _ENVISION_BANNER_ACTIVE=1

    # Save cursor, set scroll region [2..LINES] (reserves line 1),
    # move to 1;1, draw banner, restore cursor.
    # Re-setting scroll region every prompt self-heals after resize or
    # full-screen programs (vim/less) that reset terminal state.
    if [ -n "${NO_COLOR}" ]; then
        printf '\e7\e[2;%dr\e[1;1H\e[2K%s%*s\e8' "$_lines" "${_parts}" "$_pad" "" >&2
    else
        printf '\e7\e[2;%dr\e[1;1H\e[2K\e[44;1;37m%s%*s\e[0m\e8' "$_lines" "${_parts}" "$_pad" "" >&2
    fi
}
"#;

/// Bash-specific: PROMPT_COMMAND integration.
const BASH_PROMPT: &str = r#"
if [ -n "${BASH_VERSION}" ]; then
    if [[ "${PROMPT_COMMAND}" != *"_envision_banner"* ]]; then
        PROMPT_COMMAND="_envision_banner${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
    fi
fi
"#;

/// Zsh-specific: precmd hook integration.
const ZSH_PROMPT: &str = r#"
if [ -n "${ZSH_VERSION}" ]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd _envision_banner
fi
"#;

const FISH_HOOK: &str = r#"
function envision
    switch $argv[1]
        case session set unset clear profile
            set -l _envision_out (command envision $argv)
            set -l _envision_rc $status
            if test $_envision_rc -eq 0; and test -n "$_envision_out"
                eval $_envision_out
            end
            return $_envision_rc
        case '*'
            command envision $argv
    end
end

function _envision_banner --on-event fish_prompt
    test "$ENVISION_BANNER" = "off"; and return
    test -n "$TMUX"; and return
    test -z "$ENVISION_SESSION"; and test -z "$ENVISION_PROFILE"; and return

    set -l _cols $COLUMNS
    test -z "$_cols"; and set _cols 80
    set -l _lines $LINES
    test -z "$_lines"; and set _lines 24
    set -l _parts ""

    if test -n "$ENVISION_PROFILE"
        set _parts " $ENVISION_PROFILE"
    end

    if test -n "$ENVISION_SESSION_ID"
        set -l _state "clean"
        test "$ENVISION_DIRTY" = "1"; and set _state "dirty"
        set -l _tracked (test -n "$ENVISION_TRACKED"; and echo $ENVISION_TRACKED; or echo 0)
        set -l _sess "$ENVISION_SESSION_ID | $_tracked tracked | $_state"
        if test -n "$_parts"
            set _parts "$_parts | $_sess"
        else
            set _parts " $_sess"
        end
    end

    test -z "$_parts"; and return
    set _parts "$_parts "

    set -l _len (string length "$_parts")
    set -l _pad (math "$_cols - $_len")
    test $_pad -lt 0; and set _pad 0

    set -gx _ENVISION_BANNER_ACTIVE 1

    if set -q NO_COLOR
        printf '\e7\e[2;%dr\e[1;1H\e[2K%s%*s\e8' "$_lines" "$_parts" "$_pad" "" >&2
    else
        printf '\e7\e[2;%dr\e[1;1H\e[2K\e[44;1;37m%s%*s\e[0m\e8' "$_lines" "$_parts" "$_pad" "" >&2
    end
end
"#;
