# Unsloth Qwen3 Adaptation (Rust vs Iron)

This runbook adapts `Qwen3_(4B)_Instruct.ipynb` to train two comparable arms:

- Arm A: Rust target generation
- Arm B: Iron target generation

## 1) Prepare datasets

```bash
python3 scripts/generate_foundation_v1.py
python3 scripts/dataset_validator.py data/pilot/foundation_v1/manifest.v1_candidate.jsonl \
  --report-json data/pilot/foundation_v1/report.v1_candidate.json
python3 scripts/export_unsloth_dataset.py data/pilot/foundation_v1/manifest.v1_candidate.jsonl \
  --out-dir data/pilot/foundation_v1/unsloth
```

Produced files:

- `data/pilot/foundation_v1/unsloth/rust_train.jsonl`
- `data/pilot/foundation_v1/unsloth/rust_val.jsonl`
- `data/pilot/foundation_v1/unsloth/rust_test.jsonl`
- `data/pilot/foundation_v1/unsloth/iron_train.jsonl`
- `data/pilot/foundation_v1/unsloth/iron_val.jsonl`
- `data/pilot/foundation_v1/unsloth/iron_test.jsonl`

## 2) Notebook edits

Use your existing notebook setup/model-loading cells unchanged.

Replace the FineTome loading section with local JSON loading:

```python
from datasets import load_dataset

# Set one arm at a time: "rust" or "iron"
ARM = "rust"

train_file = f"data/pilot/foundation_v1/unsloth/{ARM}_train.jsonl"
val_file = f"data/pilot/foundation_v1/unsloth/{ARM}_val.jsonl"

dataset = load_dataset("json", data_files={"train": train_file, "validation": val_file})
train_dataset = dataset["train"]
val_dataset = dataset["validation"]
```

You can keep `standardize_data_formats` and chat template application:

```python
from unsloth.chat_templates import standardize_data_formats, get_chat_template

tokenizer = get_chat_template(tokenizer, chat_template="qwen3-instruct")
train_dataset = standardize_data_formats(train_dataset)
val_dataset = standardize_data_formats(val_dataset)

def formatting_prompts_func(examples):
    convos = examples["conversations"]
    texts = [tokenizer.apply_chat_template(c, tokenize=False, add_generation_prompt=False) for c in convos]
    return {"text": texts}

train_dataset = train_dataset.map(formatting_prompts_func, batched=True)
val_dataset = val_dataset.map(formatting_prompts_func, batched=True)
```

Use `train_on_responses_only` as in the original notebook.

## 3) Training config parity

Keep all hyperparameters identical between Rust and Iron runs:

- same base model
- same LoRA config
- same batch / grad accumulation
- same max steps or epochs
- same seed list

Only change `ARM` and output directory name.

## 4) Suggested run names

- `qwen3-4b-redox-rust-seed3407`
- `qwen3-4b-redox-iron-seed3407`

If possible, repeat with one extra seed (for example 1337).

## 5) Output artifacts to keep

- trainer logs and loss curves
- adapter checkpoints per arm
- exact command / notebook commit hash
- evaluation JSON (compile@1, test@1, failure categories)
