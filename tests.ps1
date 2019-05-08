cargo build

$difff = New-Object System.Collections.ArrayList

ls examples | ForEach-Object -Process {
	if ($_.Extension -ne ".wait" -and $_.Extension -ne ".output" -and $_.GetType() -eq [System.IO.FileInfo]) {
		Write-Host -NoNewline -ForegroundColor Cyan "Running ${_}".PadRight(30)
		$file = $_.BaseName
		$dirr = $_.DirectoryName
		$out = "$dirr/output/$file.output"
		
		target/debug/lambda_core -f $_.FullName > $out

		$gold = "$dirr/gold/$file.output"
		if ((Test-Path $gold) -and (Get-Content $out)) {
			if (diff (cat $out) (cat $gold)) {
				Write-Host -ForegroundColor Red "DIFFERENT"

				$a = $difff.Add(@($gold, $out, $_.FullName))
			} else {
				Write-Host -ForegroundColor Green "OK"
			}
		} else {
			target/debug/lambda_core -f $_.FullName > $gold
			Write-Host -ForegroundColor Yellow "Created"
		}
	}
}


$difff | ForEach-Object -Process {
	Write-Host ""
	Write-Host -ForegroundColor Cyan $_[2]
	Write-Host -NoNewline -ForegroundColor Green "Gold Standard".PadRight(60)
	Write-Host -ForegroundColor Red "Test Result"

	$differences = diff (cat $_[0]) (cat $_[1]) -IncludeEqual

	$differences | foreach {
		if ($_.sideindicator -eq "=>") {
			Write-Host -NoNewline -ForegroundColor Red $_.InputObject.PadRight(60)
			Write-Host -ForegroundColor Red $_.sideindicator
		} elseif ($_.sideindicator -eq "=>") {
			Write-Host -NoNewline -ForegroundColor Green $_.InputObject.PadRight(60)
			Write-Host -ForegroundColor Green $_.sideindicator
		} else {
			Write-Host -NoNewline $_.InputObject.PadRight(60)
			Write-Host $_.sideindicator
		}
	}
}
