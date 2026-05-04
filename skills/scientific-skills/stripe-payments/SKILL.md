---
name: stripe-payments
description: "Stripe payments integration: charges, subscriptions, invoices, webhooks, Connect platform, and fraud prevention (Radar). Build payment workflows, recurring billing, and marketplace payouts."
tags: [stripe, payments, billing, subscriptions, fintech, api, zorai]
---
## Overview

Stripe payments integration: charges, subscriptions, invoices, webhooks, Connect platform, and Radar fraud prevention.

## Installation

```bash
uv pip install stripe
```

## Basic Charge

```python
import stripe
stripe.api_key = "sk_test_..."

intent = stripe.PaymentIntent.create(
    amount=2000,  # $20.00
    currency="usd",
    payment_method="pm_card_visa",
    confirm=True,
)
