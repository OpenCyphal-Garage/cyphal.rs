import numpy as np
import matplotlib.pyplot as plt

MARKER = "|"


def arduino_rec_frames(plt: plt.Axes):
    y_best = [9.4, 17.6, 23.4, 29.2, 35, 40.7, 46.4, 52.2]
    y_worst = [13, 21.3, 27, 32.8, 38.7, 44.4, 50.2, 56]

    y_worst_tlsf = [
        11.282,
        19.664,
        25.141,
        30.823,
        36.505,
        42.182,
        47.952,
        53.641,
    ]
    y_best_tlsf = [
        8.429,
        16.605,
        22.223,
        27.905,
        33.588,
        39.264,
        45.035,
        50.723,
    ]

    x = [1, 2, 3, 4, 5, 6, 7, 8]

    # ax.subplot(1, 2, 2)
    plt.title.set_text("Empfangen von Rahmen")
    plt.plot(x, y_best, 'g-', marker=MARKER)
    plt.plot(x, y_worst, 'g--', marker=MARKER)
    plt.plot(x, y_best_tlsf, 'r-', marker=MARKER)
    plt.plot(x, y_worst_tlsf, 'r--', marker=MARKER)

    plt.set_xlabel("Rahmen-Anzahl")
    plt.legend(["Best Case", "Worst Case",
               "tlsf Best Case", "tlsf Worst Case"])

    diff = sum(np.array(y_worst) - np.array(y_best)) / len(x)
    elevation_from_two = sum(np.array(list(y_best[i] - y_best[i - 1]
                                           for i in range(len(y_best) - 1, 1, -1)))) / (len(y_best) - 2)
    print("arduino receive statistics:")
    print("heap alloc time: " + str(diff) + " micros")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def calc_arduino_send_without_heap(y):
    allocs = np.array([
        0.552,
        0.976,
        1.458,
        1.841,
        2.223,
        2.594,
        2.976,
        3.423,
    ])
    frees = np.array([
        0.123,
        0.211,
        0.405,
        0.488,
        0.57,
        0.682,
        0.77,
        0.858,
    ])

    heap_time = allocs + frees
    return y - heap_time


def arduino_send_transfer(plt: plt.Axes):

    y = [11.3, 22.2, 31.3, 39.8, 48.6, 57.1, 65.9, 74.4]
    y_enqueue = [7.1, 15.9, 22.5, 29.1, 35.8, 42.4, 49.1, 55.7]
    y_tlsf = np.array([9.929, 19.658, 27.035, 34.411,
                      41.411, 49.282, 56.800, 64.152])
    y_tlsf_enqueue = [6.405, 13.211, 18.958,
                      24.611, 30.423, 36.076, 41.888, 47.541]
    x = [1, 2, 3, 4, 5, 6, 7, 8]

    y_without_heap = calc_arduino_send_without_heap(y_tlsf)

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Senden einer Übertragung")
    plt.plot(x, y, 'g-', marker=MARKER)
    plt.plot(x, y_enqueue, 'g--', marker=MARKER)
    plt.plot(x, y_tlsf, 'r-', marker=MARKER)
    plt.plot(x, y_tlsf_enqueue, 'r--', marker=MARKER)
    plt.plot(x, y_without_heap, 'y-', marker=MARKER)

    plt.legend(["gesamt", "enqeueTransfer()",
               "gesamt mit tlsf", "tlsf enqeueTransfer()", "gesamt ohne heap"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y[i] - y[i - 1]
                                           for i in range(len(y) - 1, 1, -1)))) / (len(y) - 2)
    print("arduino send statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def rust_send_transfer(plt):
    y = [1.799, 5.416, 7.457, 9.568, 11.702, 13.614, 15.802, 18.024, 35.421]
    x = [1, 2, 3, 4, 5, 6, 7, 8, 16]

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Senden einer Übertragung")
    plt.plot(x, y, 'g-', marker=MARKER)

    plt.legend(["gesamt"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y[i] - y[i - 1]
                                           for i in range(len(y) - 1, 1, -1)))) / (len(y) - 2)
    print("rust send statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def rust_rec_transfer(plt):
    y_worst = [8.258, 18.270, 27.693, 32.569,
               42.339, 47.216, 52.103, 57.355, 101.706]
    y_best = [7.364, 17.652, 27.075, 31.951,
              41.721, 46.598, 51.485, 56.725, 101.105]
    y_no_realloc_worst = [7.152, 13.052, 17.928,
                          22.793, 27.639, 32.503, 37.367, 42.614, 81.209]
    y_no_realloc_best = [6.294, 12.187, 17.057,
                         21.922, 26.804, 31.632, 36.496, 41.761, 80.338]
    y_subscr_func_reimpl_worst = [6.523, 12.029,
                                  16.564, 21.057, 25.539, 30.039, 34.539, 38.679, 75.114]
    y_subscr_func_reimpl_best = [3.123, 8.570,
                                 13.099, 17.604, 22.110, 26.574, 31.074, 35.232, 71.649]
    x = [1, 2, 3, 4, 5, 6, 7, 8, 16]

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Empfangen von Rahmen")
    plt.plot(x, y_worst, 'g--', marker=MARKER)
    plt.plot(x, y_best, 'g-', marker=MARKER)
    plt.plot(x, y_no_realloc_worst, 'r--', marker=MARKER)
    plt.plot(x, y_no_realloc_best, 'r-', marker=MARKER)
    plt.plot(x, y_subscr_func_reimpl_worst, 'b--', marker=MARKER)
    plt.plot(x, y_subscr_func_reimpl_best, 'b-', marker=MARKER)

    plt.legend(["erste Messung Worst Case", "erste Messung Best Case", "erste Optimierung Worst Case",
               "zweite Optimierung Best Case", "zweite Optimierung Worst Case", "zweite Optimierung Best Case"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y_best[i] - y_best[i - 1]
                                           for i in range(len(y_best) - 1, 1, -1)))) / (len(y_best) - 2)
    print("rust receive statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def arduino_rust_diff():
    f, (ax, bx) = plt.subplots(1, 2, sharey='row')
    ax.set_ylabel("Mikrosekunden")

    y_tlsf = np.array([9.929, 19.658, 27.035, 34.411,
                      41.411, 49.282, 56.800, 64.152])
    y_arduino_send = calc_arduino_send_without_heap(y_tlsf)
    y_arduino_receive = [
        8.429,
        16.605,
        22.223,
        27.905,
        33.588,
        39.264,
        45.035,
        50.723,
    ]
    y_rust_send = [1.799, 5.416, 7.457, 9.568,
                   11.702, 13.614, 15.802, 18.024]
    y_rust_receive = [3.123, 8.570,
                      13.099, 17.604, 22.110, 26.574, 31.074, 35.232]
    x = [1, 2, 3, 4, 5, 6, 7, 8]

    # TODO - init value
    y_send_per_frame_arduino = [None] + list(
        y_arduino_send[i] - y_arduino_send[i - 1] for i in range(len(y_arduino_send) - 1, 0, -1))
    y_send_per_frame_rust = [None] + list(
        y_rust_send[i] - y_rust_send[i - 1] for i in range(len(y_rust_send) - 1, 0, -1))

    y_rec_per_frame_arduino = [None] + list(
        y_arduino_receive[i] - y_arduino_receive[i - 1] for i in range(len(y_arduino_receive) - 1, 0, -1))
    y_rec_per_frame_rust = [None] + list(
        y_rust_receive[i] - y_rust_receive[i - 1] for i in range(len(y_rust_receive) - 1, 0, -1))

    # plt.subplot(1, 2, 1)
    ax.title.set_text("Senden von Übertragungen")
    ax.plot(x, y_arduino_send, 'r-', marker=MARKER)
    ax.plot(x, y_rust_send, 'g-', marker=MARKER)

    ax.plot(x, y_send_per_frame_arduino, 'r--')
    ax.plot(x, y_send_per_frame_rust, 'g--')

    ax.legend(["Arduino (C++)", "Rust", "Arduino pro Rahmen", "Rust pro Rahmen"])

    ax.set_xlabel("Rahmen-Anzahl")

    bx.title.set_text("Empfangen von Rahmen")
    bx.plot(x, y_arduino_receive, 'r-', marker=MARKER)
    bx.plot(x, y_rust_receive, 'g-', marker=MARKER)

    bx.plot(x, y_rec_per_frame_arduino, 'r--')
    bx.plot(x, y_rec_per_frame_rust, 'g--')

    bx.legend(["Arduino (C++)", "Rust", "Arduino pro Rahmen", "Rust pro Rahmen"])

    bx.set_xlabel("Rahmen-Anzahl")


def main():
    f, (ax, bx) = plt.subplots(1, 2, sharey='row')

    ax.set_ylabel("Mikrosekunden")
    arduino_send_transfer(ax)
    arduino_rec_frames(bx)
    plt.show()

    f, (ax, bx) = plt.subplots(1, 2, sharey='row')
    ax.set_ylabel("Mikrosekunden")
    rust_send_transfer(ax)
    rust_rec_transfer(bx)

    plt.show()

    arduino_rust_diff()
    plt.show()


if __name__ == "__main__":
    main()
