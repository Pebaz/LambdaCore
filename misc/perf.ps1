$lctimes = @()
$pytimes = @()

for ($i = 0; $i -lt 100; $i += 1) {
    Write-Host "`rProgress: $i%" -NoNewline
    $lc = measure-command {.\target\release\lambda_core.exe -f examples/fib.lcore}
    $py = measure-command {python examples/fib.py}

    $lctimes += $lc.TotalMilliseconds
    $pytimes += $py.TotalMilliseconds
}

Write-Host

$sum = 0
$lctimes | Foreach { $sum += $_}
$x = $sum / $lctimes.count
Write-Host "LambdaCore avg run time: $x ms"

$sum = 0
$pytimes | Foreach { $sum += $_}
$x = $sum / $pytimes.count
Write-Host "Python average run time: $x ms"
