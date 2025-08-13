# SuperNova Fixtures — Pasta (Fib v1)

- format: `supernova_v1`
- curve: `pasta` (Pallas/Vesta)
- circuit: `FibStep [a,b] -> [b, a+b]`
- num_steps: 10
- public input[0] (hex): 9000000000000000000000000000000000000000000000000000000000000000

## Hashes
- vk.blake2b512:  7e1a8b2fb5a8302e7e693a89d2021924e14a9c39d0a9fcd177a043849f9bc56813caf792f4aaa00e46230a8412a8cc7f1f1de0513d29ee858a377d2c08b2469d
- vk.blake2b256:  15f41af42dc959885b220193021d0fc419f5e0eb9bec22ce1df701eb4746993a
- vk.sha256:      e1d98db8f7d5cfc1eb5d37b90f1f62e848e85b8e86fa2deb8b74078b98cb6725
- proof.blake2b256: 9cbb536b4aeb48fa078437dc7a3c50b9d21d5011f0f2708a52bd50deab5a8583
- proof.sha256:     71738e921ebe9072b964d6947373929d1a3d3f532ec2402635f2a72dc6c556d1

## Versions
- nova-snark = "0.41"
- base64 = "0.22"
- bincode = "1"
- ff = "0.13"
- ark-pallas = "0.4"
- ark-vesta  = "0.4"
