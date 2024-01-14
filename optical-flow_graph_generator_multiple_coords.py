import serial
import time
import matplotlib.pyplot as plt
import matplotlib.animation as animation

# 初始化参数
buffer_size = 50  # 数据缓冲区大小
optical_flow_coord_x = []  # 用于存储光流坐标x
optical_flow_coord_y = []  # 用于存储光流坐标y

# 初始化图表
fig, ax = plt.subplots()
lines_x, = ax.plot([], [], color='#ff0000', marker=".", linewidth=.5)
lines_y, = ax.plot([], [], color='#0000ff', marker=".", linewidth=.5)
ax.set_xlim(0, buffer_size)
ax.set_ylim(-800, 800)

# 初始化串口
my_camera = serial.Serial("COM3", 115200)

def animate(i):
    global optical_flow_coord_x, optical_flow_coord_y

    # 检查串口是否接收到新数据
    if my_camera.in_waiting:
        if my_camera.read() == b'w':
            code = my_camera.read(13)
            print(code)
            package = code[1:-7]
            _x = int.from_bytes(bytes([package[0], package[1]]), 'little', signed=True)
            _y = int.from_bytes(bytes([package[2], package[3]]), 'little', signed=True)

            # print(_x, _y, package[4])

            # 更新数据缓冲区
            optical_flow_coord_x.append(_x)
            optical_flow_coord_y.append(_y)
            if len(optical_flow_coord_x) > buffer_size:
                optical_flow_coord_x.pop(0)
                optical_flow_coord_y.pop(0)

            # 更新图表
            lines_x.set_data(range(len(optical_flow_coord_x)), optical_flow_coord_x)
            lines_y.set_data(range(len(optical_flow_coord_y)), optical_flow_coord_y)
    
    return lines_x, lines_y

# 创建动画
ani = animation.FuncAnimation(fig, animate, interval=10, blit=True, save_count=200)
plt.show()
