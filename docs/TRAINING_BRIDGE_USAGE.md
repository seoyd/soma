# Training Bridge Usage

Example commands:

## 1. Export or generate dataset

```bash
python /path/to/local/make_synthetic_dataset.py --out target/soma_synthetic_dataset.csv
```

## 2. Validate dataset

```bash
python /path/to/local/validate_dataset.py --input target/soma_synthetic_dataset.csv
```

## 3. Train local research model

```bash
python /path/to/local/train_tabular.py \
  --input target/soma_synthetic_dataset.csv \
  --predictions-out target/soma_predictions.csv \
  --model-card-out target/soma_model_card.md
```

## 4. Evaluate predictions in Rust

Use the generated prediction CSV with the existing Sprint 07 Rust prediction import and external evaluation path.

## 5. Compare baseline vs external

Run the Rust-side comparison report on the same folds after importing the generated predictions.

The repository no longer ships those helper scripts; bring your own local Python tools and wire them in through `training_script_path`.
