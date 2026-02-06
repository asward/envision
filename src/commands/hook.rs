use crate::cli::Shell;

pub fn run(shell: &Shell) -> Result<u8, String> {
    let code = match shell {
        Shell::Bash | Shell::Zsh => BASH_HOOK,
        Shell::Fish => FISH_HOOK,
    };
    print!("{code}");
    Ok(0)
}

const BASH_HOOK: &str = r#"
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
"#;
