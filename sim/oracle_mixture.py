"""Differential oracle for the latent-class mixture (`docs/08` REPRO-003): a SciPy
two-class fit and one-class null on `<dir>/theta.csv` and `X.csv`; writes `<dir>/oracle.csv`
as `name,value` lines."""
import os
import sys

import numpy as np
from scipy.optimize import minimize
from scipy.special import expit, logsumexp


def _dif_nll(p, th, X, mixture):
    k = X.shape[1]
    if mixture:
        eta, a, b, d = p[0], p[1:1 + k], p[1 + k:1 + 2 * k], p[1 + 2 * k:]
        pi = expit(eta)
        classes, log_w = (-1., 1.), np.log([1 - pi, pi])
    else:
        a, b, d = p[:k], p[k:2 * k], np.zeros(k)
        classes, log_w = (0.,), np.array([0.])
    ll, logits = [], []
    for z in classes:
        lo = a * (th[:, None] - b - d * z)
        ll.append((X * lo - np.logaddexp(0, lo)).sum(1))
        logits.append(lo)
    T = np.array(ll) + log_w[:, None]
    lse = logsumexp(T, axis=0)
    post = np.exp(T - lse)
    g_a, g_b, g_d = np.zeros(k), np.zeros(k), np.zeros(k)
    for c, z in enumerate(classes):
        resid = post[c][:, None] * (X - expit(logits[c]))
        g_a -= (resid * (th[:, None] - b - d * z)).sum(0)
        g_b += (resid * a).sum(0)
        g_d += (resid * a * z).sum(0)
    if mixture:
        return -lse.sum(), np.concatenate([[-(post[1] - pi).sum()], g_a, g_b, g_d])
    return -lse.sum(), np.concatenate([g_a, g_b])


def dif_fit(th, X, seed=0, starts=4):
    """One-class 2PL (null) and two-class uniform-DIF mixture; returns (null, mixture)."""
    k = X.shape[1]
    opts = {"maxiter": 5000, "gtol": 1e-8}
    null = minimize(_dif_nll, np.concatenate([np.ones(k), np.zeros(k)]), args=(th, X, False),
                    jac=True, method="L-BFGS-B", options=opts)
    rng = np.random.default_rng(seed)
    best = None
    for _ in range(starts):
        p0 = np.concatenate([[rng.normal(0, .3)], null.x[:k], null.x[k:], rng.normal(0, .5, k)])
        res = minimize(_dif_nll, p0, args=(th, X, True), jac=True, method="L-BFGS-B", options=opts)
        if best is None or res.fun < best.fun:
            best = res
    return null, best


d = sys.argv[1]
theta = np.loadtxt(os.path.join(d, "theta.csv"), delimiter=",", ndmin=1)
X = np.loadtxt(os.path.join(d, "X.csv"), delimiter=",", ndmin=2)
null, mix = dif_fit(theta, X, seed=0, starts=4)
k = X.shape[1]
with open(os.path.join(d, "oracle.csv"), "w") as f:
    f.write(f"n,{X.shape[0]}\nk,{k}\n")
    f.write(f"null_nll,{float(null.fun)!r}\nmixture_nll,{float(mix.fun)!r}\n")
    f.write(f"pi,{float(expit(mix.x[0]))!r}\n")
    for j, x in enumerate(mix.x[1 + 2 * k:]):
        f.write(f"d[{j}],{float(x)!r}\n")
print(f"null nll {null.fun:.6f}, mixture nll {mix.fun:.6f}")
