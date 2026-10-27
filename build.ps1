go vet ./...
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
go test ./...
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
go build -ldflags "-H=windowsgui -s -w" -o monitors.exe .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$env:GOOS = "darwin"
try {
	go vet ./...
	if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
	foreach ($arch in "arm64", "amd64") {
		$env:GOARCH = $arch
		go build -ldflags "-s -w" -o "monitors-mac-$arch" .
		if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
	}
} finally {
	Remove-Item Env:GOOS, Env:GOARCH -ErrorAction SilentlyContinue
}
