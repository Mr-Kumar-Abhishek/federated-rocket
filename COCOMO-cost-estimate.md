# COCOMO Cost Estimation for Federated Rocket

This document provides a cost estimation for the **federated-rocket** project using the Constructive Cost Model (COCOMO).

## Project Metrics

Based on the current source code footprint:
- **Lines of Code (LOC):** ~22,700
- **Kilo Lines of Code (KLOC):** 22.7

## Basic COCOMO Calculation

COCOMO provides different modes based on the project complexity. We will estimate using the **Organic** mode (small team, familiar with the domain) and the **Semi-detached** mode (medium-sized team, mixed experience levels).

### 1. Organic Mode
This mode is typically used for relatively small software teams developing software in a highly familiar, in-house environment.

- **Formula for Effort:** $E = 2.4 \times (KLOC)^{1.05}$
- **Formula for Time:** $T = 2.5 \times (E)^{0.38}$

**Calculations:**
- **Effort (E):** $2.4 \times (22.7)^{1.05} \approx 63.34$ Person-Months
- **Development Time (T):** $2.5 \times (63.34)^{0.38} \approx 11.68$ Months
- **Average Staffing Required:** $E / T \approx 5.4$ Developers

### 2. Semi-detached Mode
This mode is applicable for projects where the team consists of a mixture of experienced and inexperienced staff, working on a somewhat complex system.

- **Formula for Effort:** $E = 3.0 \times (KLOC)^{1.12}$
- **Formula for Time:** $T = 2.5 \times (E)^{0.35}$

**Calculations:**
- **Effort (E):** $3.0 \times (22.7)^{1.12} \approx 98.22$ Person-Months
- **Development Time (T):** $2.5 \times (98.22)^{0.35} \approx 12.33$ Months
- **Average Staffing Required:** $E / T \approx 7.9$ Developers

## Project Cost Estimation (in USD)

To estimate the total project cost in USD, we apply an average loaded monthly cost per developer. 
*Assuming an average loaded salary of **$10,000 USD** per developer per month (which includes overhead, benefits, etc.).*

| Project Mode | Person-Months | Development Time (Months) | Estimated Total Cost (USD) |
| :--- | :--- | :--- | :--- |
| **Organic** | ~63.34 | ~11.7 | **$633,400 USD** |
| **Semi-detached** | ~98.22 | ~12.3 | **$982,200 USD** |

> **Note:** This is a theoretical estimation based on the basic COCOMO model. Real-world costs may vary depending on team location, exact salary structures, tool costs, and infrastructure overhead.
