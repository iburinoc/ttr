#include <bits/stdc++.h>
using namespace std;

int main(int argc, char **argv) {
  uint32_t seed = (uint32_t)stoull(argv[1]);
  minstd_rand gen(seed);

  vector<int> tickets(46);
  iota(tickets.begin(), tickets.end(), 0);
  vector<int> bigs;

  auto erase = [&](int i) {
    tickets.erase(find(tickets.begin(), tickets.end(), i));
    bigs.push_back(i);
  };

  erase(11);
  erase(15);
  erase(16);
  erase(21);
  erase(24);
  erase(31);

  auto shuffle1 = [&](vector<int> v) {
    for (int i = 0; i < v.size() - 1; i++) {
      int idx = uniform_int_distribution<uint_fast32_t>(i, v.size() - 1)(gen);
      swap(v[idx], v[i]);
    }
    return v;
  };
  auto shuffle2 = [&](vector<int> v) {
    vector<int> o;
    while (v.size() > 1) {
      int idx = uniform_int_distribution<uint_fast32_t>(0, v.size() - 1)(gen);
      o.push_back(v[idx]);
      v.erase(v.begin() + idx);
    }
    o.push_back(v[0]);
    return o;
  };

  tickets = shuffle2(tickets);
  bigs = shuffle2(bigs);

  for (int i = 0; i < 6; i++) {
    cout << tickets[i] << " ";
  }
  cout << endl;
  for (int i = 0; i < 2; i++) {
    cout << bigs[i] << " ";
  }
  cout << endl;

  vector<int> trains(110);
  iota(trains.begin(), trains.end(), 0);
  auto pick = [&]() {
    int idx =
        uniform_int_distribution<uint_fast32_t>(0, trains.size() - 1)(gen);
    auto iter = trains.begin() + idx;
    int val = *iter;
    trains.erase(iter);
    return val;
  };
  auto pickN = [&](int n) {
    for (int j = 0; j < n; j++) {
      cout << setw(3) << pick() << " ";
    }
    cout << endl;
  };
  pickN(5);
  pickN(4);
  pickN(4);
}
