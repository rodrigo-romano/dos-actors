import numpy as np
import matplotlib.pyplot as plt

cfd = np.load("cfd2025_windloads.pkl", allow_pickle=True)


ax = plt.figure().add_subplot(projection="3d")
ax1 = plt.figure().add_subplot(projection="3d")

cats = [
    "Topend",
    "Tup",
    "Tbot",
    "M1Baffle",
    "M1cov",
    "M1covin",
    "crane",
    "arm",
    "cabletruss",
    "Cring",
    "GIR",
    "LGSS",
    "M1c_",
    "M1p",
    "M1s",
    "plat",
    "M2seg",
]

for c in cats:
    print(c)
    n = [x[1]["OSS"] for x in cfd["nodes"] if "OSS" in x[1] and c in x[0]]
    if n == []:
        continue
    xyz = np.vstack(n)
    if c.startswith("M1") or c.startswith("LGSS"):
        ax1.plot(xyz[:, 0], xyz[:, 1], xyz[:, 2], "o", label=c)
    else:
        ax.plot(xyz[:, 0], xyz[:, 1], xyz[:, 2], "o", label=c)

ax.legend()
ax1.legend()
