"""
Export ckiplab/bert-tiny-chinese to ONNX for Migao V1.0 neural reranker.

Requirements:
    pip install torch transformers onnx onnxruntime

Output:
    models/bert-tiny-chinese.onnx
    models/vocab.txt

Run from repo root:
    python scripts/export_reranker.py
"""

import os
import torch
from transformers import BertForMaskedLM, BertTokenizer

MODEL_NAME = "ckiplab/bert-tiny-chinese"
OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "models")


def export():
    os.makedirs(OUT_DIR, exist_ok=True)

    print(f"Downloading {MODEL_NAME} ...")
    tokenizer = BertTokenizer.from_pretrained(MODEL_NAME)
    model = BertForMaskedLM.from_pretrained(MODEL_NAME)
    model.eval()

    # Save vocab.txt
    vocab_path = os.path.join(OUT_DIR, "vocab.txt")
    tokenizer.save_vocabulary(OUT_DIR)
    print(f"Vocab saved → {vocab_path}")

    # Dummy input for tracing (batch=1, seq_len=16)
    seq_len = 16
    dummy_ids   = torch.ones(1, seq_len, dtype=torch.long)
    dummy_mask  = torch.ones(1, seq_len, dtype=torch.long)
    dummy_types = torch.zeros(1, seq_len, dtype=torch.long)

    onnx_path = os.path.join(OUT_DIR, "bert-tiny-chinese.onnx")
    torch.onnx.export(
        model,
        (dummy_ids, dummy_mask, dummy_types),
        onnx_path,
        input_names=["input_ids", "attention_mask", "token_type_ids"],
        output_names=["logits"],
        dynamic_axes={
            "input_ids":      {0: "batch", 1: "seq"},
            "attention_mask": {0: "batch", 1: "seq"},
            "token_type_ids": {0: "batch", 1: "seq"},
            "logits":         {0: "batch", 1: "seq"},
        },
        opset_version=14,
    )
    print(f"Model saved → {onnx_path}")

    # Quick sanity check
    import onnxruntime as ort
    sess = ort.InferenceSession(onnx_path)
    out = sess.run(None, {
        "input_ids":      dummy_ids.numpy(),
        "attention_mask": dummy_mask.numpy(),
        "token_type_ids": dummy_types.numpy(),
    })
    assert out[0].shape == (1, seq_len, tokenizer.vocab_size), \
        f"Unexpected output shape: {out[0].shape}"
    print(f"Sanity check passed — logits shape: {out[0].shape}")
    print("Done. Copy models/ to %APPDATA%\\Migao\\models\\ to activate neural reranker.")


if __name__ == "__main__":
    export()
