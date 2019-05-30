def fib(n):
    a = 0
    b = 1
    if n < 0:
        return 'Invalid Input'
    elif n == 0:
        return a
    elif n == 1:
        return b
    else:
        for i in range(2, n):
            c = a + b
            a = b
            b = c
        return b


result = fib(40)
print(f'fib(40) = {result}')
