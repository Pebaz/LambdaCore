ls examples | ForEach-Object -Process {
	if ($_.Extension -ne ".output" -and $_.GetType() -eq [System.IO.FileInfo]) {
		Write-Host -ForegroundColor Green "Running ${_}"
		$file = $_.BaseName
		$dirr = $_.DirectoryName
		target/release/lambda_core -f $_.FullName > "$dirr/output/$file.output"
	}
}
