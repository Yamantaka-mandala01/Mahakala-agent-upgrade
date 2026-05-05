import base64
with open('mahakala_pixel.png', 'rb') as f:
    data = f.read()
b64 = base64.b64encode(data).decode()
print(b64)
