# Enrollment TLS fixtures

These DER files are public test material, including the deliberately committed private key. They are not deployment credentials and must never be installed as a trusted CA on a device.

The CA signs a leaf for `alpha.tail-test.ts.net` and `beta.tail-test.ts.net`, valid for ten years from November 2026. Enrollment tests add this CA only to their private test client, exercise TLS on ephemeral loopback ports, and also require rejection with the default roots and a mismatched hostname. The production build has no test-CA configuration or TLS-verification bypass.
