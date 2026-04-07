$containerName = "surrealdb_dev"
$dataPath = "C:/SurrealDB/data"
$hostPort = "8000"
$containerPort = "8000"
$user = "root"
$pass = "root"

# Extract SurrealDB version from Cargo.toml
$version = (Select-String -Path "Cargo.toml" -Pattern '(?<=surrealdb.*version = ")[^"]+').Matches.Value

if (-not $version) { 
    Write-Host "Error: Could not find version in Cargo.toml" -ForegroundColor Red
    pause; exit 
}

# Run SurrealDB in Docker
Write-Host "Starting SurrealDB v$version on port $hostPort..." -ForegroundColor Cyan

docker run --rm --pull always `
  --name $containerName `
  -p "${hostPort}:${containerPort}" `
  -v "${dataPath}:/data" `
  "surrealdb/surrealdb:v$version" `
  start --log trace --user $user --pass $pass "surrealkv:/data/TravelRS.db"