"""Differential oracle for the bridging fit (`docs/08` REPRO-003): a NumPy/SciPy fit with
analytic gradients on `<dir>/R.csv`, `mask.csv` and `weights.csv`; writes `<dir>/oracle.csv`
as `name,value` lines."""
import os
import sys

import numpy as np
from scipy.optimize import minimize


def bridging_fit(R, mask, lam_b=0.15, lam_f=0.03, seed=0, weights=None):
    """Weighted bridging objective of docs/02 §A (d = 1), mu unpenalized.
    Returns (mu, b_u, b_j, f_u, f_j) and the SciPy result object."""
    n, m = R.shape
    rng = np.random.default_rng(seed)
    idx = np.where(mask)
    obs = R[idx]
    w = np.ones(n) if weights is None else np.asarray(weights, float)
    wo = w[idx[0]]
    x0 = np.concatenate([[obs.mean()], rng.normal(0, .1, n), rng.normal(0, .1, m),
                         rng.normal(0, .3, n), rng.normal(0, .3, m)])

    def unpack(x):
        return x[0], x[1:1 + n], x[1 + n:1 + n + m], x[1 + n + m:1 + 2 * n + m], x[1 + 2 * n + m:]

    def cost_grad(x):
        mu, bu, bj, fu, fj = unpack(x)
        e = mu + bu[idx[0]] + bj[idx[1]] + fu[idx[0]] * fj[idx[1]] - obs
        cost = (wo * e ** 2).sum() + lam_b * (bu @ bu + bj @ bj) + lam_f * (fu @ fu + fj @ fj)
        g = 2 * wo * e
        grad = np.concatenate([[g.sum()],
                               np.bincount(idx[0], g, n) + 2 * lam_b * bu,
                               np.bincount(idx[1], g, m) + 2 * lam_b * bj,
                               np.bincount(idx[0], g * fj[idx[1]], n) + 2 * lam_f * fu,
                               np.bincount(idx[1], g * fu[idx[0]], m) + 2 * lam_f * fj])
        return cost, grad

    res = minimize(cost_grad, x0, jac=True, method="L-BFGS-B",
                   options={"maxiter": 20000, "gtol": 1e-10, "ftol": 1e-15, "maxcor": 20})
    return unpack(res.x), res


def bridging_multistart(R, mask, starts=8, **kw):
    """Lowest objective over `starts` seeded starts (as the Rust engine does, T48)."""
    best = None
    for s in range(starts):
        params, res = bridging_fit(R, mask, seed=s, **kw)
        if best is None or res.fun < best[1].fun:
            best = (params, res)
    return best


d = sys.argv[1]
R = np.loadtxt(os.path.join(d, "R.csv"), delimiter=",", ndmin=2)
mask = np.loadtxt(os.path.join(d, "mask.csv"), delimiter=",", ndmin=2) > 0.5
weights = np.loadtxt(os.path.join(d, "weights.csv"), delimiter=",", ndmin=1)
(mu, bu, bj, fu, fj), res = bridging_multistart(R, mask, starts=8, weights=weights)
with open(os.path.join(d, "oracle.csv"), "w") as f:
    f.write(f"objective,{float(res.fun)!r}\n")
    f.write(f"mu,{float(mu)!r}\n")
    for name, v in (("b_j", bj), ("f_j", fj), ("b_u", bu), ("f_u", fu)):
        for k, x in enumerate(v):
            f.write(f"{name}[{k}],{float(x)!r}\n")
print(f"objective {res.fun:.10f} after {res.nit} iterations ({res.message})")
