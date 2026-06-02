# Smart Home Agent Eval 20260529_015327

成功标准：严格参数（工具名、action、关键数值、设备类别、场景名、可提取楼层/房间/设备名）。

ROB_STATE: `/mnt/emmc/lxm/ROB/test/logs/smart_home_eval_20260529_015327/state`

## test1.md

- Success: 108 / 155
- Accuracy: 69.68%
- Log: `/mnt/emmc/lxm/ROB/test/logs/smart_home_eval_20260529_015327/test1.jsonl`

| Group | Success | Total | Accuracy |
|---:|---:|---:|---:|
| 1 | 0 | 5 | 0.00% |
| 2 | 3 | 5 | 60.00% |
| 3 | 0 | 5 | 0.00% |
| 4 | 1 | 5 | 20.00% |
| 5 | 0 | 5 | 0.00% |
| 6 | 3 | 5 | 60.00% |
| 7 | 5 | 5 | 100.00% |
| 8 | 5 | 5 | 100.00% |
| 9 | 3 | 5 | 60.00% |
| 10 | 5 | 5 | 100.00% |
| 11 | 1 | 5 | 20.00% |
| 12 | 4 | 5 | 80.00% |
| 13 | 5 | 5 | 100.00% |
| 14 | 4 | 5 | 80.00% |
| 15 | 5 | 5 | 100.00% |
| 16 | 4 | 5 | 80.00% |
| 17 | 5 | 5 | 100.00% |
| 18 | 5 | 5 | 100.00% |
| 19 | 5 | 5 | 100.00% |
| 20 | 5 | 5 | 100.00% |
| 21 | 3 | 5 | 60.00% |
| 22 | 4 | 5 | 80.00% |
| 23 | 4 | 5 | 80.00% |
| 24 | 3 | 5 | 60.00% |
| 25 | 5 | 5 | 100.00% |
| 26 | 3 | 5 | 60.00% |
| 27 | 2 | 5 | 40.00% |
| 28 | 5 | 5 | 100.00% |
| 29 | 1 | 5 | 20.00% |
| 30 | 5 | 5 | 100.00% |
| 31 | 5 | 5 | 100.00% |

## test2.md

- Success: 177 / 310
- Accuracy: 57.10%
- Log: `/mnt/emmc/lxm/ROB/test/logs/smart_home_eval_20260529_015327/test2.jsonl`

| Group | Success | Total | Accuracy |
|---:|---:|---:|---:|
| 1 | 2 | 10 | 20.00% |
| 2 | 7 | 10 | 70.00% |
| 3 | 0 | 10 | 0.00% |
| 4 | 6 | 10 | 60.00% |
| 5 | 0 | 10 | 0.00% |
| 6 | 7 | 10 | 70.00% |
| 7 | 9 | 10 | 90.00% |
| 8 | 8 | 10 | 80.00% |
| 9 | 4 | 10 | 40.00% |
| 10 | 10 | 10 | 100.00% |
| 11 | 1 | 10 | 10.00% |
| 12 | 7 | 10 | 70.00% |
| 13 | 9 | 10 | 90.00% |
| 14 | 3 | 10 | 30.00% |
| 15 | 9 | 10 | 90.00% |
| 16 | 3 | 10 | 30.00% |
| 17 | 9 | 10 | 90.00% |
| 18 | 10 | 10 | 100.00% |
| 19 | 8 | 10 | 80.00% |
| 20 | 8 | 10 | 80.00% |
| 21 | 5 | 10 | 50.00% |
| 22 | 5 | 10 | 50.00% |
| 23 | 3 | 10 | 30.00% |
| 24 | 5 | 10 | 50.00% |
| 25 | 6 | 10 | 60.00% |
| 26 | 3 | 10 | 30.00% |
| 27 | 2 | 10 | 20.00% |
| 28 | 4 | 10 | 40.00% |
| 29 | 4 | 10 | 40.00% |
| 30 | 10 | 10 | 100.00% |
| 31 | 10 | 10 | 100.00% |

## Failure Reason Histogram

| Reason | Count |
|---|---:|
| delta_percent missing | 59 |
| device_name | 56 |
| action | 19 |
| volume_percent missing | 15 |
| no_tool_call | 8 |
| delta_kelvin missing | 6 |
| device_category | 5 |
| room | 4 |
| floor | 3 |
| wrong_tool | 3 |
| color_temperature_kelvin missing | 1 |
| color_name missing | 1 |

## Failure Samples

- `test1.md` g1 q1: 音量调大。
  - reason: no_tool_call; process_exit=1
  - calls: `[]`
- `test1.md` g1 q2: 扬声器音量调大。
  - reason: delta_percent missing expected=10; device_name expected=扬声器 actual=None
  - calls: `[{"id": "mL1fhbxDYBSz4aGr6O5OkQMcgXKfjIQD", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test1.md` g1 q3: 增大音量。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "7fvE9x7HRXwBSboE1hVDpHQxzX8XumDY", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "增大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"增大扬声器音量\"}"}]`
- `test1.md` g1 q4: 提高扬声器音量。
  - reason: delta_percent missing expected=10; device_name expected=扬声器 actual=None
  - calls: `[{"id": "dQWktM0ecXpXBVB5DGGjE4B75GMY0iUl", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "提高扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"提高扬声器音量\"}"}]`
- `test1.md` g1 q5: 把音量调大。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "dG2XnFSU3EdbrABhNn4SiBw1OD7zYmAZ", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test1.md` g2 q2: 扬声器音量调大20%。
  - reason: device_name expected=扬声器 actual=None
  - calls: `[{"id": "Dr34S1qKwRSW0Q5R61Fc4WG4LvbrFjIk", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量", "delta_percent": 20}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\",\"delta_percent\":20}"}]`
- `test1.md` g2 q4: 增加10%的音量。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "uowFTIWtPe22JuPEhrsVHSm7bTnzH6cw", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "增加扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"增加扬声器音量\"}"}]`
- `test1.md` g3 q1: 音量调小。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "MZFyUfskBoSJSD3KOpre5SH8vHC5QJ5M", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "调小扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"调小扬声器音量\"}"}]`
- `test1.md` g3 q2: 扬声器音量调小。
  - reason: delta_percent missing expected=10; device_name expected=扬声器 actual=None
  - calls: `[{"id": "yVwa82J6dpeuPWzonbPMSymPtPHhqVXb", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test1.md` g3 q3: 减小音量。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "UucgCfVgW8cSbSrnaon8XjxsgXvRGe5f", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test1.md` g3 q4: 降低扬声器音量。
  - reason: delta_percent missing expected=10; device_name expected=扬声器 actual=None
  - calls: `[{"id": "amYCzwkfHENTrez5qtcbBGSc7kkISJQV", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test1.md` g3 q5: 把音量调小。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "wBob5c2E9UIKCtuLf46mbFP3MTdr3Nv2", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test1.md` g4 q1: 音量调小20%。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "es5xQuqKsKGUD5QEvofaNhqtBv8HLYlr", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "调小扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"调小扬声器音量\"}"}]`
- `test1.md` g4 q2: 扬声器音量调小10%。
  - reason: delta_percent missing expected=10; device_name expected=扬声器 actual=None
  - calls: `[{"id": "nzWT7wYI0lcOlKAbCFO4CX23Xa6UUYQl", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "调小扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"调小扬声器音量\"}"}]`
- `test1.md` g4 q4: 减少20%的音量。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "PTS3ctLnJLcIWDJ8j8AbWqLBJN0fO21a", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量 20%"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量 20%\"}"}]`
- `test1.md` g4 q5: 音量向下调节10%。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "uFgZtbJk4fo91BprLeEXzeb7HovDMmlS", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test1.md` g5 q1: 扬声器调到静音。
  - reason: volume_percent missing expected=0; device_name expected=扬声器 actual=None
  - calls: `[{"id": "ygCexnC2XkYHbV5zP7fEKjpF2rIpjuJO", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "将扬声器调至静音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"将扬声器调至静音\"}"}]`
- `test1.md` g5 q2: NAS调到静音。
  - reason: volume_percent missing expected=0; device_name expected=NAS actual=None
  - calls: `[{"id": "gQHxNN6UpoIfU2yySbGtTs2vwCZYn7Wl", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "将NAS音量调至静音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"将NAS音量调至静音\"}"}]`
- `test1.md` g5 q3: 设置为静音。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "QcTrsI5uIgdC8uNwA8PfJGPHDMoQUEYz", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "设置扬声器静音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"设置扬声器静音\"}"}]`
- `test1.md` g5 q4: 打开静音模式。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "9Zv1UDeh3o1mVXvnl7vQMT82oLSHpVrb", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "打开静音模式"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"打开静音模式\"}"}]`
- `test1.md` g5 q5: 扬声器静音。
  - reason: volume_percent missing expected=0; device_name expected=扬声器 actual=None
  - calls: `[{"id": "02W5eIfPzBeyIqWaH312Zt3RSttQD7LF", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "扬声器静音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"扬声器静音\"}"}]`
- `test1.md` g6 q2: 扬声器音量设置到50%。
  - reason: device_name expected=扬声器 actual=None
  - calls: `[{"id": "iMRU2d3AUWRg3GOEIP9TauA7GUZVv7Vn", "name": "smart_home_control_speaker", "arguments": {"action": "set_volume", "tool_title": "设置扬声器音量至50%", "volume_percent": 50}, "raw_arguments": "{\"action\":\"set_volume\",\"tool_title\":\"设置扬声器音量至50%\",\"volume_percent\":50}"}]`
- `test1.md` g6 q5: 扬声器音量调为60%。
  - reason: device_name expected=扬声器 actual=None
  - calls: `[{"id": "SbSmvP6a3qSghJn3UTsHw4PKYpVeNw50", "name": "smart_home_control_speaker", "arguments": {"action": "set_volume", "tool_title": "设置扬声器音量为 60%", "volume_percent": 60}, "raw_arguments": "{\"action\":\"set_volume\",\"tool_title\":\"设置扬声器音量为 60%\",\"volume_percent\":60}"}]`
- `test1.md` g9 q1: 把一楼的客厅的主灯亮度调大。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "r55fuqvptPuAmMk00e26W9YrP64O2XIc", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调大客厅主灯亮度", "room": "客厅", "device_name": "主灯", "floor": "一楼"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调大客厅主灯亮度\",\"room\":\"客厅\",\"device_name\":\"主灯\",\"floor\":\"一楼\"}"}]`
- `test1.md` g9 q2: 一楼的客厅的主灯调亮一点。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "YEkaT0InLHii4EuVWpZ9QiQYkQmLjHkT", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调亮一楼客厅主灯", "room": "客厅", "floor": "一楼", "device_name": "主灯"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调亮一楼客厅主灯\",\"room\":\"客厅\",\"floor\":\"一楼\",\"device_name\":\"主灯\"}"}]`
- `test1.md` g11 q1: 把一楼的客厅的主灯亮度调小。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "wUlhCDRpx083R9JM1v2Ew9mUoGCDOUh1", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调小客厅主灯亮度", "room": "客厅", "device_name": "主灯", "floor": "一楼"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调小客厅主灯亮度\",\"room\":\"客厅\",\"device_name\":\"主灯\",\"floor\":\"一楼\"}"}]`
- `test1.md` g11 q2: 把二楼的书房的台灯调暗一点。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "ij3gw9VbIBudMxlRLjIJ0qcFgWVbLLyc", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调暗二楼书房台灯", "room": "书房", "floor": "二楼", "device_name": "台灯"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调暗二楼书房台灯\",\"room\":\"书房\",\"floor\":\"二楼\",\"device_name\":\"台灯\"}"}]`
- `test1.md` g11 q4: 调小一楼的餐厅的吊灯亮度。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "A1TWRR3MyqfRkFlnKovGs8sVcqjvmC5C", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调小餐厅吊灯亮度", "room": "餐厅", "floor": "一楼", "device_name": "吊灯"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调小餐厅吊灯亮度\",\"room\":\"餐厅\",\"floor\":\"一楼\",\"device_name\":\"吊灯\"}"}]`
- `test1.md` g11 q5: 把二楼的儿童房的吸顶灯调暗。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "LyDdSdLxMCF7oQblcfVeH9sWVIB1u4r3", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调暗二楼儿童房吸顶灯", "room": "儿童房", "floor": "二楼", "device_name": "吸顶灯"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调暗二楼儿童房吸顶灯\",\"room\":\"儿童房\",\"floor\":\"二楼\",\"device_name\":\"吸顶灯\"}"}]`
- `test1.md` g12 q4: 二楼的儿童房的吸顶灯亮度下调15%。
  - reason: no_tool_call; process_exit=1
  - calls: `[]`
- `test1.md` g14 q3: 把三楼的主卧的床头灯调成冷光。
  - reason: action expected=increase_color_temperature actual=set_light_tone; delta_kelvin missing expected=500
  - calls: `[{"id": "sJqJnE4YYQvcW3JspNpmVAP7jlbU2hdr", "name": "smart_home_control_light_temperature", "arguments": {"action": "set_light_tone", "tool_title": "设置三楼主卧床头灯为冷光", "light_tone": "cool", "color_temperature_kelvin": 6000, "device_name": "床头灯", "room": "主卧", "floor": "三楼"}, "raw_arguments": "{\"action\":\"set_light_tone\",\"tool_title\":\"设置三楼主卧床头灯为冷光\",\"light_tone\":\"cool\",\"color_temperature_kelvin\":6000,\"device_name\":\"床头灯\",\"room\":\"主卧\",\"floor\":\"三楼\"}"}]`
- `test1.md` g16 q3: 三楼的主卧的床头灯调成暖光。
  - reason: action expected=decrease_color_temperature actual=set_light_tone; delta_kelvin missing expected=500
  - calls: `[{"id": "04fmM9bBooUHZNuqcaF0fWIrsoYJVCiD", "name": "smart_home_control_light_temperature", "arguments": {"action": "set_light_tone", "tool_title": "设置三楼主卧床头灯为暖光", "light_tone": "warm", "color_temperature_kelvin": 3000, "device_name": "床头灯", "room": "主卧", "floor": "三楼"}, "raw_arguments": "{\"action\":\"set_light_tone\",\"tool_title\":\"设置三楼主卧床头灯为暖光\",\"light_tone\":\"warm\",\"color_temperature_kelvin\":3000,\"device_name\":\"床头灯\",\"room\":\"主卧\",\"floor\":\"三楼\"}"}]`
- `test1.md` g21 q2: 打开三楼的主卧的窗帘。
  - reason: device_name expected=窗帘 actual=None
  - calls: `[{"id": "8HtDkpqAkYhhrwGgWM5a3gI7QO5o1bh1", "name": "smart_home_control_curtain", "arguments": {"action": "open", "tool_title": "打开三楼主卧窗帘", "room": "主卧", "floor": "三楼"}, "raw_arguments": "{\"action\":\"open\",\"tool_title\":\"打开三楼主卧窗帘\",\"room\":\"主卧\",\"floor\":\"三楼\"}"}]`
- `test1.md` g21 q4: 把二楼的儿童房的窗帘打开。
  - reason: device_name expected=窗帘 actual=None
  - calls: `[{"id": "wjXr71Menq3KlBL0LdmVusbhvaYc91iA", "name": "smart_home_control_curtain", "arguments": {"action": "open", "tool_title": "打开二楼儿童房窗帘", "room": "儿童房", "floor": "二楼"}, "raw_arguments": "{\"action\":\"open\",\"tool_title\":\"打开二楼儿童房窗帘\",\"room\":\"儿童房\",\"floor\":\"二楼\"}"}]`
- `test1.md` g22 q5: 收起一楼的餐厅的纱帘。
  - reason: action expected=close actual=stop
  - calls: `[{"id": "iF4F1Ap9Vl85uDeMgSdCsYLmzNQxaLlt", "name": "smart_home_control_curtain", "arguments": {"action": "stop", "tool_title": "停止一楼餐厅纱帘", "device_name": "纱帘", "room": "餐厅", "floor": "一楼"}, "raw_arguments": "{\"action\":\"stop\",\"tool_title\":\"停止一楼餐厅纱帘\",\"device_name\":\"纱帘\",\"room\":\"餐厅\",\"floor\":\"一楼\"}"}]`
- `test1.md` g23 q4: 把二楼的儿童房的窗帘停止。
  - reason: device_name expected=窗帘 actual=None
  - calls: `[{"id": "Dfi3rp1Th7pLHphSebwczz03Mjk2HjVX", "name": "smart_home_control_curtain", "arguments": {"action": "stop", "tool_title": "停止二楼儿童房窗帘", "room": "儿童房", "floor": "二楼"}, "raw_arguments": "{\"action\":\"stop\",\"tool_title\":\"停止二楼儿童房窗帘\",\"room\":\"儿童房\",\"floor\":\"二楼\"}"}]`
- `test1.md` g24 q3: 三楼的主卧的窗帘开合度调节到30%。
  - reason: device_name expected=窗帘 actual=None
  - calls: `[{"id": "FQjRIxbji8MWq1hBcTolrTtM1oYLwwR7", "name": "smart_home_control_curtain", "arguments": {"action": "set_position", "tool_title": "设置三楼主卧窗帘开合度30%", "position_percent": 30, "room": "主卧", "floor": "三楼"}, "raw_arguments": "{\"action\":\"set_position\",\"tool_title\":\"设置三楼主卧窗帘开合度30%\",\"position_percent\":30,\"room\":\"主卧\",\"floor\":\"三楼\"}"}]`
- `test1.md` g24 q4: 把二楼的儿童房的窗帘设置到70%。
  - reason: device_name expected=窗帘 actual=None
  - calls: `[{"id": "jlb2YpTMC0RnN6P4nU0pzj8QlooJGX7I", "name": "smart_home_control_curtain", "arguments": {"action": "set_position", "tool_title": "设置二楼儿童房窗帘开合度", "position_percent": 70, "room": "儿童房", "floor": "二楼"}, "raw_arguments": "{\"action\":\"set_position\",\"tool_title\":\"设置二楼儿童房窗帘开合度\",\"position_percent\":70,\"room\":\"儿童房\",\"floor\":\"二楼\"}"}]`
- `test1.md` g26 q4: 把三楼的主卧的墙壁插座打开。
  - reason: device_category expected=outlet actual=wall_switch
  - calls: `[{"id": "DM9Zr3QrTYcSsCfxr0gnKy3OhLAjZDP1", "name": "smart_home_control_power", "arguments": {"action": "turn_on", "device_category": "wall_switch", "tool_title": "打开三楼主卧墙壁插座", "device_name": "墙壁插座", "floor": "三楼", "room": "主卧"}, "raw_arguments": "{\"action\":\"turn_on\",\"device_category\":\"wall_switch\",\"tool_title\":\"打开三楼主卧墙壁插座\",\"device_name\":\"墙壁插座\",\"floor\":\"三楼\",\"room\":\"主卧\"}"}]`
- `test1.md` g26 q5: 启动二楼的儿童房的智能插头。
  - reason: device_name expected=智能插头 actual=智能插座
  - calls: `[{"id": "di4Tgjn5AbMfBRdqsuXlsmrJmas4SHxi", "name": "smart_home_control_power", "arguments": {"action": "turn_on", "device_category": "outlet", "tool_title": "启动二楼儿童房智能插头", "device_name": "智能插座", "floor": "二楼", "room": "儿童房"}, "raw_arguments": "{\"action\":\"turn_on\",\"device_category\":\"outlet\",\"tool_title\":\"启动二楼儿童房智能插头\",\"device_name\":\"智能插座\",\"floor\":\"二楼\",\"room\":\"儿童房\"}"}]`
- `test1.md` g27 q3: 断开一楼的餐厅的智能插座。
  - reason: floor expected=一楼 actual=None; room expected=餐厅 actual=None; device_name expected=智能插座 actual=None
  - calls: `[{"id": "k9iw9KapV3J5EFFusDmHD0MGnu7Psk12", "name": "smart_home_control_power", "arguments": {"action": "turn_off", "device_category": "outlet", "tool_title": "关闭一楼餐厅智能插座"}, "raw_arguments": "{\"action\":\"turn_off\",\"device_category\":\"outlet\",\"tool_title\":\"关闭一楼餐厅智能插座\"}"}]`
- `test1.md` g27 q4: 把三楼的主卧的墙壁插座关闭。
  - reason: device_category expected=outlet actual=wall_switch
  - calls: `[{"id": "4J3v0okE5JzIARNR1FXPKDxiwy0TBp72", "name": "smart_home_control_power", "arguments": {"action": "turn_off", "device_category": "wall_switch", "tool_title": "关闭三楼主卧墙壁插座", "device_name": "墙壁插座", "floor": "三楼", "room": "主卧"}, "raw_arguments": "{\"action\":\"turn_off\",\"device_category\":\"wall_switch\",\"tool_title\":\"关闭三楼主卧墙壁插座\",\"device_name\":\"墙壁插座\",\"floor\":\"三楼\",\"room\":\"主卧\"}"}]`
- `test1.md` g27 q5: 停止二楼的儿童房的智能插头。
  - reason: device_name expected=智能插头 actual=None
  - calls: `[{"id": "1h6k8B2L7Po07hirYoIkL4oSxdISruHX", "name": "smart_home_control_power", "arguments": {"action": "turn_off", "device_category": "outlet", "tool_title": "关闭二楼儿童房智能插座", "room": "儿童房", "floor": "二楼"}, "raw_arguments": "{\"action\":\"turn_off\",\"device_category\":\"outlet\",\"tool_title\":\"关闭二楼儿童房智能插座\",\"room\":\"儿童房\",\"floor\":\"二楼\"}"}]`
- `test1.md` g29 q1: 关闭一楼的客厅的墙壁开关。
  - reason: no_tool_call; process_exit=1
  - calls: `[]`
- `test1.md` g29 q3: 把三楼的主卧的墙壁开关关闭。
  - reason: device_name expected=墙壁开关 actual=None
  - calls: `[{"id": "lr9oMl7QI3c0ApIUplweB2wUwkEVLEBT", "name": "smart_home_control_power", "arguments": {"action": "turn_off", "device_category": "wall_switch", "tool_title": "关闭三楼主卧墙壁开关", "room": "主卧", "floor": "三楼"}, "raw_arguments": "{\"action\":\"turn_off\",\"device_category\":\"wall_switch\",\"tool_title\":\"关闭三楼主卧墙壁开关\",\"room\":\"主卧\",\"floor\":\"三楼\"}"}]`
- `test1.md` g29 q4: 断开二楼的儿童房的单开开关。
  - reason: floor expected=二楼 actual=None; room expected=儿童房 actual=None; device_name expected=单开 actual=None
  - calls: `[{"id": "l2DVO3K5oFFeaMDIIzk3MCJbal7GtHTL", "name": "smart_home_control_power", "arguments": {"action": "turn_off", "device_category": "wall_switch", "tool_title": "关闭二楼儿童房单开开关"}, "raw_arguments": "{\"action\":\"turn_off\",\"device_category\":\"wall_switch\",\"tool_title\":\"关闭二楼儿童房单开开关\"}"}]`
- `test1.md` g29 q5: 停止一楼的餐厅的双开开关。
  - reason: action expected=turn_off actual=stop
  - calls: `[{"id": "tP62wjY33usKX4M7CUkwFpa5tQhFRPUh", "name": "smart_home_control_power", "arguments": {"action": "stop", "device_category": "wall_switch", "tool_title": "停止一楼餐厅双开开关", "device_name": "双开", "room": "餐厅", "floor": "一楼"}, "raw_arguments": "{\"action\":\"stop\",\"device_category\":\"wall_switch\",\"tool_title\":\"停止一楼餐厅双开开关\",\"device_name\":\"双开\",\"room\":\"餐厅\",\"floor\":\"一楼\"}"}]`
- `test2.md` g1 q1: 声音太小了，稍微开大点儿。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "Szv7qPw6ozySSdEZsmGG7d9tb92D5lkQ", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g1 q2: 根本听不清啊，把音量加大。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "VtDj7NYUbMZkISVXchodZzuPolpf6ilR", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g1 q3: 帮我把那个…音箱声音弄大点。
  - reason: delta_percent missing expected=10; device_name expected=音箱 actual=None
  - calls: `[{"id": "oEtuFkXYggxKFQQgAs3epFktdRhJMULP", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大音箱音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大音箱音量\"}"}]`
- `test2.md` g1 q4: 声音给点力啊，那个播放器。
  - reason: delta_percent missing expected=10; device_name expected=播放器 actual=None
  - calls: `[{"id": "Ob8L8Yes4VwoE2HTmOQfF6ug4HGHlXY5", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g1 q5: 稍微响一点吧，这声音太小了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "bzkxam7DbLQAzWQKOfTneoERzSxhsh1y", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g1 q7: 调大点呗，那个音响。
  - reason: delta_percent missing expected=10; device_name expected=音响 actual=None
  - calls: `[{"id": "5r7bkxZEpJsvTtYFfUZyfJyWT6FwLEtO", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大音响音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大音响音量\"}"}]`
- `test2.md` g1 q9: 加点音量，这啥也听不见。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "F3iZdclnBa2IQICWDUJTDC5RqfJ3hdFL", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "增加扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"增加扬声器音量\"}"}]`
- `test2.md` g1 q10: 往上调调声音，太轻了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "PNH864cyy7YtL7T9K09DqlkXyig6OF7r", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g2 q4: 稍微亮一点嗓门，加百分之十。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "eHexsg1XdewPwgJozHtCRbrKVrLc6C9g", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\"}"}]`
- `test2.md` g2 q5: 往上走百分之十五啊，那个音响。
  - reason: delta_percent missing expected=15; device_name expected=音响 actual=None
  - calls: `[{"id": "9UUp2ZgWDVncYcsuqED8TK4yijDmy5GG", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大音响音量"}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大音响音量\"}"}]`
- `test2.md` g2 q9: 往上加点啊，百分之二十。
  - reason: wrong_tool expected=smart_home_control_speaker actual=['smart_home_control_light']
  - calls: `[{"id": "2Hetdxq2kHiEpGxWpGzG7GO8UcM8zgMD", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调高灯光亮度", "delta_percent": 20}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调高灯光亮度\",\"delta_percent\":20}"}]`
- `test2.md` g3 q1: 哎呀吵死了，声音关小点。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "b31k3ceSfrxQErTyi53lnS0wODDRW7EC", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q2: 声音太大了，赶紧减一点。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "bu83U8zQrcYeraqEA2ZUDyMe1FtaVPWl", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q3: 帮我把那个…声音稍微弄轻点。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "1S77uiKLQYiFqO9XMILVsazs63JcYJT0", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q4: 把那个音响声音往下拉一拉。
  - reason: delta_percent missing expected=10; device_name expected=音响 actual=None
  - calls: `[{"id": "WCqA3Rzf7BwJZY1JpFmvO3Wf7hoFajE3", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低音响音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低音响音量\"}"}]`
- `test2.md` g3 q5: 这也太吵了，音量调低点吧。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "hse0eh2RMwXxNTWjzfUhNfgOwH0ePw8I", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q6: 稍微轻一点儿呢，震耳朵了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "dAy9k4Kn1B1Kovbabw80SL9lJVQE7kD6", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q7: 小点声小点声，打扰到我了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "2lhhzl1zY9XBO7EYTUroX4nPVzxM8slA", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q8: 关小点呗，那个声音。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "L8wZaseIJXmiVbsTHTq3OsV8Gwid5kAX", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q9: 声音弄小点啊，我要睡觉了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "sdrQirZ4TzzR60b14XdqQQugBkvSzwTG", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g3 q10: 调低点吧，这动静太大了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "ZIrv3joIPWRt0vo6yFenvyVATMI7Dhl7", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g4 q1: 声音稍微小一格，太吵了。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "MstXmu4WWm3aIGwgwa6al2DEEYxlMU99", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g4 q2: 帮我把音量减个百分之十。
  - reason: delta_percent missing expected=10
  - calls: `[{"id": "zKAgFzl5XQXf4ZnZab45BHqdXKmXVDpb", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量\"}"}]`
- `test2.md` g4 q5: 往下压百分之二十啊，那个音响。
  - reason: delta_percent missing expected=20; device_name expected=音响 actual=None
  - calls: `[{"id": "q5lnjs8idzsWYJT8w1ACVmU20ibqkZjJ", "name": "smart_home_control_speaker", "arguments": {"action": "decrease_volume", "tool_title": "降低扬声器音量 20%"}, "raw_arguments": "{\"action\":\"decrease_volume\",\"tool_title\":\"降低扬声器音量 20%\"}"}]`
- `test2.md` g4 q7: 小百分之二十吧，震耳朵。
  - reason: action expected=decrease_volume actual=increase_volume
  - calls: `[{"id": "2W628OadnA3Xx079DqZar3ACOMMm4CQ1", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "调大扬声器音量", "delta_percent": 20}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"调大扬声器音量\",\"delta_percent\":20}"}]`
- `test2.md` g5 q1: 别出声了，直接静音吧。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "ywQ8EM7Qi3X9TUyeGDZvZi6j027JAzcT", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g5 q2: 吵死了，把声音给切了。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "wh9TnWMoSJl00ByBURiU4J4tq8bCIpZw", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g5 q3: 帮我把那个…音响弄成静音。
  - reason: volume_percent missing expected=0; device_name expected=音响 actual=None
  - calls: `[{"id": "W2YztC2RrU1or6ZioUJEg5NTKpPrmOoi", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "将音响设置为静音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"将音响设置为静音\"}"}]`
- `test2.md` g5 q4: 让那个播放器别响了。
  - reason: volume_percent missing expected=0; device_name expected=播放器 actual=None
  - calls: `[{"id": "cumQSd16QJvoWS4bFqKQZi20TjWkbz0N", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g5 q5: 哎，把声音关死吧。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "EXGv554SXSAWcMSy2VWjsBIUy4YS24OD", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "关闭扬声器声音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"关闭扬声器声音\"}"}]`
- `test2.md` g5 q6: 闭嘴啦，那个音箱。
  - reason: volume_percent missing expected=0; device_name expected=音箱 actual=None
  - calls: `[{"id": "fnMVRF4SkHLMTMfbVBDHlcqPiLpUpKem", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "关闭音箱"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"关闭音箱\"}"}]`
- `test2.md` g5 q7: 静音静音，接个电话。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "JnS61pu2DOMq41nRuxsevjh4QLUU4jmZ", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g5 q8: 关掉声音呗，别放了。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "6FSbfoEVxw7M7AEgzzAxuCe2YBWA5iqc", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "关闭扬声器声音"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"关闭扬声器声音\"}"}]`
- `test2.md` g5 q9: 给它静音了啊，那个设备。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "tylhRtW4Spgci3Uz9M7Nzl2afmxxp0Mn", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g5 q10: 别让它出声了，吵得头疼。
  - reason: volume_percent missing expected=0
  - calls: `[{"id": "yRduCI4bGZBfcgZTxWVLWffuQKxisoXn", "name": "smart_home_control_speaker", "arguments": {"action": "mute", "tool_title": "静音扬声器"}, "raw_arguments": "{\"action\":\"mute\",\"tool_title\":\"静音扬声器\"}"}]`
- `test2.md` g6 q5: 弄到百分之五十啊，那个音响。
  - reason: device_name expected=音响 actual=None
  - calls: `[{"id": "EqEE7YKI5ktFMYmp3OUBY5Os9esV9Ufw", "name": "smart_home_control_speaker", "arguments": {"action": "set_volume", "tool_title": "设置音响音量 50%", "volume_percent": 50}, "raw_arguments": "{\"action\":\"set_volume\",\"tool_title\":\"设置音响音量 50%\",\"volume_percent\":50}"}]`
- `test2.md` g6 q9: 给弄到一半的音量呗。
  - reason: action expected=set_volume actual=increase_volume; volume_percent missing expected=50
  - calls: `[{"id": "94P5gupxTQyEjMRWo2xf7OcaFP2Sw7ES", "name": "smart_home_control_speaker", "arguments": {"action": "increase_volume", "tool_title": "增加扬声器音量", "delta_percent": 10}, "raw_arguments": "{\"action\":\"increase_volume\",\"tool_title\":\"增加扬声器音量\",\"delta_percent\":10}"}]`
- `test2.md` g6 q10: 直接切到百分之三十，那个音箱。
  - reason: device_name expected=音箱 actual=None
  - calls: `[{"id": "KoXWbthS4yiaLi36Jt6xLTtb6qwLVqXU", "name": "smart_home_control_speaker", "arguments": {"action": "set_volume", "tool_title": "设置音箱音量至30%", "volume_percent": 30}, "raw_arguments": "{\"action\":\"set_volume\",\"tool_title\":\"设置音箱音量至30%\",\"volume_percent\":30}"}]`
- `test2.md` g7 q10: 开灯开灯，啥也看不见了。
  - reason: no_tool_call
  - calls: `[]`
- `test2.md` g8 q8: 帮我熄个灯呗，那个台灯。
  - reason: device_name expected=台灯 actual=None
  - calls: `[{"id": "TzFfAY6qwj6pnn0HVh9oMN6glPMPltz5", "name": "smart_home_control_light", "arguments": {"action": "turn_off", "tool_title": "关闭台灯"}, "raw_arguments": "{\"action\":\"turn_off\",\"tool_title\":\"关闭台灯\"}"}]`
- `test2.md` g8 q10: 别开着了，那几个射灯。
  - reason: device_name expected=射灯 actual=None
  - calls: `[{"id": "abMs4UaZBBmA7U9qMOYJGGno3zEhBZvT", "name": "smart_home_control_light", "arguments": {"action": "turn_off", "tool_title": "关闭射灯"}, "raw_arguments": "{\"action\":\"turn_off\",\"tool_title\":\"关闭射灯\"}"}]`
- `test2.md` g9 q2: 看不清了，把卧室的灯弄亮些。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "9Pn8BVDVdJqGY9eb9wvRfChRS6YEDjrS", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调亮卧室灯光", "room": "主卧"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调亮卧室灯光\",\"room\":\"主卧\"}"}]`
- `test2.md` g9 q3: 帮我把那个…台灯亮度往上加加。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "aNhmvLZoxhhBN06rR6wWmwVRfrtI0Cab", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调高台灯亮度", "device_name": "台灯"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调高台灯亮度\",\"device_name\":\"台灯\"}"}]`
- `test2.md` g9 q5: 哎，让那个吸顶灯亮堂一点。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "M5eHEjGkCr4ksFO5ZRtevrdDKerEJlpb", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调亮吸顶灯亮度", "device_name": "吸顶灯"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调亮吸顶灯亮度\",\"device_name\":\"吸顶灯\"}"}]`
- `test2.md` g9 q6: 提提亮度吧，这屋太黑了。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "EnmSL7MpwiMt37sLTWR8Oc1aOxziCJWq", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调亮全屋灯光"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调亮全屋灯光\"}"}]`
- `test2.md` g9 q7: 调亮点呗，书房这个灯。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "AadGw8koYNcLZSLoQx94oRIXE6TUfIpz", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "调亮书房灯光", "room": "书房"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"调亮书房灯光\",\"room\":\"书房\"}"}]`
- `test2.md` g9 q9: 加点亮度，我都看不见字了。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "MSgZsOTdqn5c0dvQIYll9BzwoocBu1bp", "name": "smart_home_control_light", "arguments": {"action": "increase_brightness", "tool_title": "增加灯光亮度"}, "raw_arguments": "{\"action\":\"increase_brightness\",\"tool_title\":\"增加灯光亮度\"}"}]`
- `test2.md` g11 q1: 刺眼死了，亮度往下压一压。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "kDsDKaZvCsNWF7tjS6mBTkLAMVbUVYUA", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "降低灯光亮度"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"降低灯光亮度\"}"}]`
- `test2.md` g11 q2: 太亮了晃眼，把客厅的灯弄暗点。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "HsXr9k26VE85adomv40VbzFMBDCPyuF0", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调暗客厅灯光", "room": "客厅"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调暗客厅灯光\",\"room\":\"客厅\"}"}]`
- `test2.md` g11 q3: 帮我把那个…床头灯稍微调暗些。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "p4L7cnI9oHYUlCW1749dbfgUyvAYU55O", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调暗床头灯亮度", "device_name": "床头灯"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调暗床头灯亮度\",\"device_name\":\"床头灯\"}"}]`
- `test2.md` g11 q4: 卧室灯光太强了，减一点亮度。
  - reason: delta_percent missing expected=20; device_name expected=卧室灯 actual=None
  - calls: `[{"id": "uwcmb9dqFzsPJ7Iac0RzWxGN6dc0geVy", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调低卧室灯光亮度", "room": "主卧"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调低卧室灯光亮度\",\"room\":\"主卧\"}"}]`
- `test2.md` g11 q5: 哎，让那个吸顶灯暗下来一点。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "8oiFRufMoVoArphnPh9i01vd2TF6jVcL", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "调暗吸顶灯亮度", "device_name": "吸顶灯"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"调暗吸顶灯亮度\",\"device_name\":\"吸顶灯\"}"}]`
- `test2.md` g11 q6: 降降亮度吧，这屋太亮了。
  - reason: delta_percent missing expected=20
  - calls: `[{"id": "7iYwn0MXsIbsrFxyIr6d6XlhMDXNHzJd", "name": "smart_home_control_light", "arguments": {"action": "decrease_brightness", "tool_title": "降低灯光亮度"}, "raw_arguments": "{\"action\":\"decrease_brightness\",\"tool_title\":\"降低灯光亮度\"}"}]`
