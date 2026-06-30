param(
  [ValidateSet("json", "table")]
  [string]$Format = "table",
  [string]$Write = ""
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$py = Join-Path $scriptDir "discover-agents.py"

if ($Write) {
  python $py --format $Format --write $Write
} else {
  python $py --format $Format
}
