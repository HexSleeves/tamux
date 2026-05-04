---
name: medical-coding
description: "Medical code mapping and classification tools. ICD-10-CM/PCS, CPT, SNOMED CT, HCPCS, LOINC, RxNorm. Code validation, mapping between terminologies, HCC risk adjustment, and reimbursement modeling."
tags: [icd10, cpt, snomed, loinc, medical-coding, hcc, reimbursement, zorai]
---
## Overview

Medical code mapping and classification: ICD-10-CM/PCS, CPT, SNOMED CT, HCPCS, LOINC, RxNorm. Code validation, cross-terminology mapping, HCC risk adjustment, and reimbursement modeling.

## HCC Risk Adjustment

```python
# HCC mapping (simplified)
hcc_map = {
    "E11.9": "HCC 19",    # Diabetes without complications
    "I10": "HCC 134",     # Essential hypertension
    "N18.3": "HCC 138",   # CKD stage 3
}

def get_hcc_codes(dx_codes):
    codes = set()
    for code in dx_codes:
        if code in hcc_map:
            codes.add(hcc_map[code])
    return list(codes)
```

## Workflow

1. Map source codes to standard terminologies (ICD-10, SNOMED, LOINC)
2. Validate codes against official code sets
3. Cross-map between terminologies for interoperability
4. Calculate HCC risk scores for reimbursement modeling
5. Generate compliance-ready code lists for submissions
