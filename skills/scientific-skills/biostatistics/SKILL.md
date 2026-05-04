---
name: biostatistics
description: "Medical biostatistics hypothesis testing toolkit: t-tests, ANOVA, chi-square, Fisher exact, Mann-Whitney, Kruskal-Wallis, sample size calculation, power analysis, multiple testing correction, survival analysis, and clinical trial biostatistics."
tags: [biostatistics, hypothesis-testing, clinical-trials, statistics, sample-size, power-analysis, zorai]
---
## Overview

Medical biostatistics hypothesis testing: t-tests, ANOVA, chi-square, Fisher exact, Mann-Whitney, Kruskal-Wallis, sample size calculation, power analysis, multiple testing correction, and clinical trial biostatistics.

## Installation

```bash
uv pip install scipy statsmodels
```

## Common Tests

```python
import numpy as np
from scipy import stats

# Two-sample t-test
group_a = np.random.normal(100, 15, 50)
group_b = np.random.normal(110, 15, 50)
t_stat, p_val = stats.ttest_ind(group_a, group_b)

# Mann-Whitney U (non-parametric)
u_stat, p_val = stats.mannwhitneyu(group_a, group_b)

# Chi-square
observed = np.array([[30, 10], [20, 40]])
chi2, p_val, dof, expected = stats.chi2_contingency(observed)
