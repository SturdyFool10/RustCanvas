$buildType = Read-Host -Prompt "Select build type: (1) Regular, (2) AVX2, (3) AVX512"

switch ($buildType) {
    "1" {
        Write-Host "Building with regular settings..."
        cargo build --release
    }
    "2" {
        Write-Host "Building with AVX2 instructions..."
        $env:RUSTFLAGS = "-C target-feature=+avx2,+fma,+bmi,+bmi2,+popcnt,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2"
        cargo build --release
        $env:RUSTFLAGS = ""
    }
    "3" {
        Write-Host "Building with AVX512 instructions..."
        $env:RUSTFLAGS = "-C target-feature=+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx2,+fma,+bmi,+bmi2,+popcnt,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2"
        cargo build --release
        $env:RUSTFLAGS = ""
    }
    default {
        Write-Host "Building with regular settings..."
        cargo build --release
        Write-Host "Building with AVX2 instructions..."
        $env:RUSTFLAGS = "-C target-feature=+avx2,+fma,+bmi,+bmi2,+popcnt,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2"
        cargo build --release
        $env:RUSTFLAGS = ""
        Write-Host "Building with AVX512 instructions..."
        $env:RUSTFLAGS = "-C target-feature=+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx2,+fma,+bmi,+bmi2,+popcnt,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2"
        cargo build --release
        $env:RUSTFLAGS = ""
        exit 1
    }
}
