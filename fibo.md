# Comparaison des vitesses de build de release et debug

    cargo build --release 0.38s
    cargo build 0.17s

# observation de fibo(47) et fibo(48)

    fibo(47) = 2971215073
    fibo(48) = 512559680

    overflow! current u32 can't handle fibo(48)

## Solution:
    saturating_add
    checked_add + unwrap