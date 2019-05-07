ls examples | ForEach-Object -Process {
	Write-Host -ForegroundColor Green "Running ${_}"
	target/debug/lambda_core -f $_.FullName
}
