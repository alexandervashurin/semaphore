param(
    [string]$BaseUrl = "http://localhost:3000/api",
    [string]$Token = "",
    [int]$Iterations = 30,
    [string]$Namespace = "default",
    [string]$ClusterId = ""
)

$ErrorActionPreference = "Stop"

function Invoke-Endpoint {
    param(
        [string]$Name,
        [string]$Url,
        [hashtable]$Headers,
        [int]$Iterations
    )

    $samples = @()
    for ($i = 0; $i -lt $Iterations; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        try {
            Invoke-WebRequest -UseBasicParsing -Uri $Url -Headers $Headers | Out-Null
        } catch {
            Write-Host "[$Name] request failed: $($_.Exception.Message)" -ForegroundColor Red
            continue
        } finally {
            $sw.Stop()
        }
        $samples += $sw.Elapsed.TotalMilliseconds
    }

    if ($samples.Count -eq 0) {
        return [PSCustomObject]@{
            Endpoint = $Name
            Count = 0
            P50Ms = [double]::NaN
            P95Ms = [double]::NaN
            MeanMs = [double]::NaN
        }
    }

    $sorted = $samples | Sort-Object
    $p50Index = [Math]::Floor(($sorted.Count - 1) * 0.50)
    $p95Index = [Math]::Floor(($sorted.Count - 1) * 0.95)

    return [PSCustomObject]@{
        Endpoint = $Name
        Count = $sorted.Count
        P50Ms = [Math]::Round($sorted[$p50Index], 2)
        P95Ms = [Math]::Round($sorted[$p95Index], 2)
        MeanMs = [Math]::Round((($sorted | Measure-Object -Average).Average), 2)
    }
}

$headers = @{}
if ($Token) {
    $headers["Authorization"] = "Bearer $Token"
}

$targets = @(
    @{ Name = "pods-list"; Url = "$BaseUrl/kubernetes/namespaces/$Namespace/pods?limit=200" },
    @{ Name = "deployments-list"; Url = "$BaseUrl/kubernetes/namespaces/$Namespace/deployments?limit=200" },
    @{ Name = "daemonsets-list"; Url = "$BaseUrl/kubernetes/daemonsets?namespace=$Namespace&limit=200" },
    @{ Name = "statefulsets-list"; Url = "$BaseUrl/kubernetes/statefulsets?namespace=$Namespace&limit=200" },
    @{ Name = "replicasets-list"; Url = "$BaseUrl/kubernetes/replicasets?namespace=$Namespace&limit=200" },
    @{ Name = "events-list"; Url = "$BaseUrl/kubernetes/k8s-events?namespace=$Namespace&limit=200" }
)

if ($ClusterId) {
    $targets += @(
        @{ Name = "pods-list-cluster"; Url = "$BaseUrl/kubernetes/clusters/$ClusterId/namespaces/$Namespace/pods?limit=200" },
        @{ Name = "deployments-list-cluster"; Url = "$BaseUrl/kubernetes/clusters/$ClusterId/namespaces/$Namespace/deployments?limit=200" }
    )
}

Write-Host "Benchmark started. Iterations per endpoint: $Iterations" -ForegroundColor Cyan
$results = foreach ($target in $targets) {
    Invoke-Endpoint -Name $target.Name -Url $target.Url -Headers $headers -Iterations $Iterations
}

$results = $results | Sort-Object -Property P95Ms -Descending
$results | Format-Table -AutoSize

Write-Host ""
Write-Host "Top slow endpoints by p95:" -ForegroundColor Yellow
$results | Select-Object -First 3 Endpoint, P95Ms, P50Ms, MeanMs | Format-Table -AutoSize
