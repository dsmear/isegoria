"""A3 (`docs/18`): the latent histogram's map from logits to a standardized distribution, as
`scoring::latent::Grid::shape` computes it; its rank, and how flat the grid moments are for the
response patterns of a few 2PL items at a smooth and at a rough histogram. numpy only."""
from itertools import product

import numpy as np


def shape(logits, theta):
    """Weights (softmax), and nodes standardized by the weights' own grid mean and variance."""
    w = np.exp(logits - logits.max())
    w /= w.sum()
    mean = (w * theta).sum()
    var = (w * (theta - mean) ** 2).sum()
    return w, (theta - mean) / np.sqrt(var)


def jacobian(f, x, eps=1e-6):
    columns = []
    for i in range(x.size):
        step = np.zeros_like(x)
        step[i] = eps
        columns.append((f(x + step) - f(x - step)) / (2 * eps))
    return np.array(columns).T


def patterns(logits, theta, a, b):
    """Probability of every response pattern of the items `(a, b)` under the histogram."""
    w, u = shape(logits, theta)
    p = 1 / (1 + np.exp(-a[None, :] * (u[:, None] - b[None, :])))
    out = []
    for x in product([0, 1], repeat=a.size):
        x = np.array(x)
        out.append((w * np.prod(np.where(x == 1, p, 1 - p), axis=1)).sum())
    return np.array(out)


def main():
    nodes3 = np.array([-1.0, 0.0, 1.0])
    for weights in ([0.25, 0.5, 0.25], [0.125, 0.75, 0.125]):
        w, u = shape(np.log(weights), nodes3)
        print(f"3 nodes, weights {weights}: nodes {np.round(u, 6)}, fourth moment "
              f"{(w * u ** 4).sum():.6f}")

    theta = np.linspace(-5.0, 5.0, 41)
    rng = np.random.default_rng(18)
    logits = rng.normal(size=theta.size)
    jac = jacobian(lambda v: np.concatenate(shape(v, theta)), logits)
    sv = np.linalg.svd(jac, compute_uv=False)
    print(f"logits -> (weights, nodes), 41 nodes: rank {(sv > 1e-7 * sv[0]).sum()}, "
          f"smallest singular values {sv[-2]:.2e}, {sv[-1]:.2e}")

    a = np.array([1.0, 1.3, 0.8, 1.5, 1.1, 0.9, 1.2, 1.4])
    b = np.linspace(-1.5, 1.5, 8)
    for name, start in (("smooth", -0.5 * theta ** 2),
                        ("rough", rng.normal(size=theta.size) - 0.5 * theta ** 2)):
        jac = jacobian(lambda v: patterns(v, theta, a, b), start)
        w, _ = shape(start, theta)
        rates = []
        for power in (1, 2, 3):
            d = theta ** power - (w * theta ** power).sum()
            rates.append(np.linalg.norm(jac @ d) / np.sqrt((w * d ** 2).sum()))
        print(f"{name} histogram, 8 items: pattern change per unit tilt along θ, θ², θ³ = "
              + ", ".join(f"{r:.2e}" for r in rates))
        jac = jacobian(lambda v: patterns(v[:41], theta, v[41:49], v[49:]),
                       np.concatenate([start, a, b]))
        sv = np.linalg.svd(jac, compute_uv=False)
        print(f"  with the 16 item parameters, singular values over the largest: "
              f"{(sv > 1e-6 * sv[0]).sum()} above 1e-6, {(sv > 1e-9 * sv[0]).sum()} above "
              f"1e-9, of {sv.size}; smallest {sv[-1] / sv[0]:.1e}")


if __name__ == "__main__":
    main()
