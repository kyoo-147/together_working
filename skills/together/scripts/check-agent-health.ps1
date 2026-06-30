param(
  [string]$Agent = ""
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$py = Join-Path $scriptDir "discover-agents.py"
$json = python $py --format json | ConvertFrom-Json

if (-not $Agent) {
  $json.agents | Select-Object display_name, status, command, path
  exit 0
}

$match = $json.agents | Where-Object { $_.id -eq $Agent -or $_.command -eq $Agent -or $_.display_name -eq $Agent } | Select-Object -First 1
if (-not $match) {
  Write-Error "Agent not found: $Agent"
  exit 1
}

$match | ConvertTo-Json -Depth 8
