# Timeout Cleanup Consistency

Cleanup counts record whether timeout exits leave lingering cargo or rustc processes. Cleanup consistency is not a test pass signal.

When real Sprint 116 observations are enabled, cleanup consistency uses post-observation process counts. Otherwise it remains carried-forward baseline evidence.
