# Training Data Artifact Population

Sprint 80 populates the materialized training-data registry with **references to existing local artifacts**.

- registry population adds dataset/version/lineage/source-class references only
- prediction CSV, model card, evaluation, and committee-pack entries point to local files only
- placeholder-safe semantics remain intact: no fake data availability is introduced
- lineage and source-class metadata stay preserved so prototype artifacts remain auditable

