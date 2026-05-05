import os, subprocess
path = os.path.dirname(os.path.abspath('mahakala_pixel.png'))
print("文件夹路径:", path)
# 用资源管理器打开文件夹
subprocess.run(['explorer', path])
