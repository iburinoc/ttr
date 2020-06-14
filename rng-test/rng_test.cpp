#include <bits/stdc++.h>
using namespace std;

int main(int argc, char **argv) {
  uint32_t seed = (uint32_t)stoull(argv[1]);
  minstd_rand g(seed);

  for (int i = 0; i < 200; i++) {
    auto gen = g;
    gen.discard(i);
    vector<int> v(110);
    iota(v.begin(), v.end(), 0);
    cout << i << ": ";
    for (int j = 0; j < 13; j++) {
      int idx = uniform_int_distribution<uint_fast32_t>(0, v.size() - 1)(gen);
      auto iter = v.begin() + idx;
      int val = *iter;
      v.erase(iter);
      cout << setw(3) << val << " ";
    }
    cout << endl;
  }
}
