ls examples | ForEach-Object -Process {
	if ($_.Extension -ne ".output") {
		Write-Host -ForegroundColor Green "Running ${_}"
		$file = $_.BaseName
		$dirr = $_.DirectoryName
		target/release/lambda_core -f $_.FullName >> "$dirr/$file.output"
	}
}
