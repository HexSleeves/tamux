---
name: aml-compliance
description: "Anti-Money Laundering (AML) and Know Your Customer (KYC) compliance workflow. Sanctions screening, PEP detection, transaction monitoring, suspicious activity reporting (SAR), and OFAC compliance."
tags: [aml, kyc, compliance, anti-money-laundering, sanctions, regulatory, zorai]
---
## Overview

Anti-Money Laundering (AML) and Know Your Customer (KYC) compliance: sanctions screening, PEP detection, transaction monitoring, and suspicious activity reporting (SAR).

## Transaction Monitoring

```python
def screen_transaction(tx):
    alerts = []
    if tx.amount > 10000:
        alerts.append("Currency Transaction Report required")
    if tx.amount > 5000 and tx.is_cash:
        alerts.append("Structuring review: cash transaction over $5k")
    if tx.origin_country in HIGH_RISK_JURISDICTIONS:
        alerts.append("High-risk jurisdiction -- enhanced due diligence")
    return alerts
