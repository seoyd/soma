# Dashboard Open and Serve

## dashboard-open
`dashboard-open` resolves only generated local HTML under the configured output root. In this sprint it prints the local file path and suggested OS open command without launching a browser in tests.

## dashboard-serve
Deferred for safety. Static local render + dashboard-open is enough for Sprint 54. A future server must remain localhost-only, GET-only, static, read-only, and must never execute commands or expose trade/order/account controls.
