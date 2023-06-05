import numpy as np
import torchtext.vocab as vocab
import queue
from matplotlib import pyplot as plt
from scipy.cluster.hierarchy import dendrogram, linkage, distance
from functools import reduce

def clustering(table):
    glove = vocab.GloVe("6B", 50, cache="../")
    A = []
    for row in table:
        line = np.array([])
        for item in row:
            vec = glove.get_vecs_by_tokens(item, lower_case_backup=True)
            line = np.append(line, np.array(vec))
            break
        A.append(line)
    
    max_n = 0
    for item in A:
        max_n = max(max_n, len(item))
    
    for i in range(len(A)):
        A[i] = np.append(A[i], np.zeros(max_n - len(A[i])))
    
    Z = linkage(A, 'ward')
    fig = plt.figure(figsize=(25, 10))
    dn = dendrogram(Z)
    plt.show()
    
    psi = {}
    dist_table = {}
    tree = {}
    
    for i in range(len(Z) + 1):
        psi[i] = set({i})
        

    for i in range(len(Z)):
        c1, c2, dist, size = Z[i, 0], Z[i, 1], Z[i, 2], Z[i, 3]
        c1, c2 = int(c1), int(c2)
        size = int(size)
        dist_table[i + len(Z) + 1] = dist
        tree[i + len(Z) + 1] = (c1, c2)
        # print(i + len(Z) + 1, c1, c2)
    
    def back(idx):
        if idx < len(Z) + 1:
            return
        back(tree[idx][0])
        back(tree[idx][1])
        psi[idx] = psi[tree[idx][0]] | psi[tree[idx][1]]
    
    back(len(Z) + len(Z))
    
    nodes = list(psi.keys())
    nodes = sorted(nodes, key=lambda x: 0 if x <= len(Z) else dist_table[x], reverse=True)
    
    cnt = 5
    res = []
    save_nodes = set()
    for i in range(len(nodes)):
        save_nodes.add(nodes[i])
        save_nodes.add(tree[nodes[i]][0])
        save_nodes.add(tree[nodes[i]][1])
        if len(save_nodes) >= cnt:
            break
    
    q = queue.Queue()
    q.put(len(Z) + len(Z))
    while not q.empty():
        top = q.get()
        if top in save_nodes:
            res.append(psi[top])
        if len(res) >= cnt:
            break
        if tree[top][0]:
            q.put(tree[top][0])
        if tree[top][1]:
            q.put(tree[top][1])
        
    return res


    