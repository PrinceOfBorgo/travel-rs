# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# Name assigned to the running Docker container
$containerName = "surrealdb_dev"

# Host directory mounted into the container at /data, where SurrealDB persists its database files.
$dataPath = "C:/SurrealDB/data"

# Port exposed on the host machine; the bot connects to SurrealDB via this port.
$hostPort = "8000"

# Port SurrealDB listens on inside the container (default: 8000).
$containerPort = "8000"

# Root credentials for the SurrealDB instance (development credentials only).
$user = "root"
$pass = "root"

# Full path to the Docker Desktop executable.
# Launched automatically when the Docker daemon is not already running.
$dockerPath = "$env:ProgramFiles\Docker\Docker\Docker Desktop.exe"

# ---------------------------------------------------------------------------
# Execution
# ---------------------------------------------------------------------------

# Extract SurrealDB version from Cargo.toml
$version = (Select-String -Path "Cargo.toml" -Pattern '(?<=surrealdb.*version = ")[^"]+').Matches.Value

if (-not $version) { 
    Write-Host "Error: Could not find version in Cargo.toml" -ForegroundColor Red
    pause; exit 
}

# Ensure Docker Desktop is running
docker info 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "Docker is not running. Starting Docker Desktop..." -ForegroundColor Yellow

    if (-not (Test-Path $dockerPath)) {
        Write-Host "Error: Docker Desktop not found at '$dockerPath'." -ForegroundColor Red
        pause; exit
    }

    Start-Process -FilePath $dockerPath

    # Wait for the Docker daemon to become responsive
    $timeoutSeconds = 120
    $elapsed = 0
    while ($true) {
        Start-Sleep -Seconds 2
        $elapsed += 2
        docker info 2>$null | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Docker is ready." -ForegroundColor Green
            break
        }
        if ($elapsed -ge $timeoutSeconds) {
            Write-Host "Error: Timed out waiting for Docker to start." -ForegroundColor Red
            pause; exit
        }
        Write-Host "Waiting for Docker... ($elapsed s)" -ForegroundColor DarkGray
    }
}

# Run SurrealDB in Docker
Write-Host "Starting SurrealDB v$version on port $hostPort..." -ForegroundColor Cyan

docker run --rm --pull always `
  --name $containerName `
  -p "${hostPort}:${containerPort}" `
  -v "${dataPath}:/data" `
  "surrealdb/surrealdb:v$version" `
  start --log trace --user $user --pass $pass "surrealkv:/data/TravelRS.db"
