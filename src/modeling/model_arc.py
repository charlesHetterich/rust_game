from typing import Iterable
import numpy as np
import torch
import torch.nn as nn
import torch.utils.tensorboard as tb

class LinearBlock(nn.Module):
    def __init__(self, in_f: int, out_f: int, activation: nn.Module):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(in_f, out_f),
            activation(),
            nn.Linear(out_f, out_f),
            activation(),
        )
        self.res_connect = nn.Linear(in_f, out_f)

    def forward(self, x: torch.Tensor):
        return self.net(x) + self.res_connect(x)

class MLP(nn.Module):
    def __init__(self, layers: list[int], activation: nn.Module = nn.ReLU):
        super().__init__()
        assert len(layers) >= 2, "Must be at least 2 Linear layers"

        L = []
        c = layers[0]
        for l in layers[1:-1]:
            L.append(LinearBlock(c, l, activation=activation))
            c = l
        L.append(nn.Linear(c, layers[-1]))
        self.net = nn.Sequential(*L)

    def forward(self, x: torch.Tensor):
        return self.net(x)

class BallPolicy(nn.Module):
    def __init__(self, n_balls: int, nb_features: int, n_actions: int, n_layers: int = 2, mlp_ratio: float = 4):
        """
        n_balls: the number of balls in the game (excluding player ball)
        nb_features: the number of features for each ball
        n_actions: the number of possible actions
        mlp_ratio: the ratio of hidden layer size to state space dimensions
        """
        super().__init__()
        state_dims = (n_balls + 1) * nb_features
        hidden_size = int(state_dims*mlp_ratio)
        self.pi = nn.Sequential(
            # MLP([state_dims, hidden_size, hidden_size, n_actions]),
            MLP([state_dims] + [hidden_size]*n_layers + [n_actions]),
            # nn.Softmax(dim=-1)
        )

    def forward(self, s: torch.Tensor):
        return self.pi(s)

# class PiApproximationWithNN():
#     def __init__(self,
#                  state_dims,
#                  num_actions,
#                  alpha):
#         """
#         state_dims: the number of dimensions of state space
#         action_dims: the number of possible actions
#         alpha: learning rate
#         """
#         self.num_actions = num_actions
#         self.pi = nn.Sequential(
#             MLP([state_dims, 32, 32, num_actions]),
#             nn.Softmax(dim=-1)
#         )
#         self.optimizer = torch.optim.Adam(self.pi.parameters(), lr=10e-6)

#     def __call__(self,s) -> int:
#         s = torch.tensor(s).float()
#         return np.random.choice(self.num_actions, p=self.pi(s).detach().numpy())

#     def update(self, s, a, gamma_t, delta):
#         """
#         s: state S_t
#         a: action A_t
#         gamma_t: gamma^t
#         delta: G-v(S_t,w)
#         """
#         s = torch.tensor(s).float()
#         a = torch.tensor(a)
#         loss = - self.pi(s)[a].log() *  delta * gamma_t #nn.CrossEntropyLoss()(self.pi(s), a) * delta * gamma_t
#         loss.backward()
#         self.optimizer.step()
#         self.optimizer.zero_grad()

# build model into jit when this file is run
if __name__ == "__main__":
    model = BallPolicy(n_balls=50, nb_features=4, n_actions=4, n_layers=40)
    model = torch.jit.script(model)
    torch.jit.save(model, "ball_policy.pt")