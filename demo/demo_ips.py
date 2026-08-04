from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from demo.mac_call_edge_sandbox import call_ips_file

result = call_ips_file()

for message in result.stdout:
    print(message.sequence)
    print(message.level)
    print(message.frame_url)
    print(message.text)
    print(message.arguments)

    if message.text.startswith("TextEncoder.prototype.encode"):
        input_value = message.arguments[1].value[0].value
        encoded_bytes = message.arguments[2].value

        print("输入：", input_value)
        print("编码结果：", encoded_bytes)
