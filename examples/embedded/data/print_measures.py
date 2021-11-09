import numpy as np
import matplotlib.pyplot as plt


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
    plt.plot(x, y_best, 'g')
    plt.plot(x, y_worst, 'g--')
    plt.plot(x, y_best_tlsf, 'r')
    plt.plot(x, y_worst_tlsf, 'r--')

    plt.set_xlabel("Rahmen-Anzahl")
    plt.legend(["Best Case", "Worst Case",
               "tlsf Best Case", "tlsf Worst Case"])

    diff = sum(np.array(y_worst) - np.array(y_best)) / len(x)
    elevation_from_two = sum(np.array(list(y_best[i] - y_best[i - 1]
                                           for i in range(len(y_best) - 1, 1, -1)))) / (len(y_best) - 2)
    print("arduino receive statistics:")
    print("heap alloc time: " + str(diff) + " micros")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def arduino_send_transfer(plt: plt.Axes):

    y = [11.3, 22.2, 31.3, 39.8, 48.6, 57.1, 65.9, 74.4]
    y_enqueue = [7.1, 15.9, 22.5, 29.1, 35.8, 42.4, 49.1, 55.7]
    y_tlsf = np.array([9.929, 19.658, 27.035, 34.411,
                      41.411, 49.282, 56.800, 64.152])
    y_tlsf_enqueue = [6.405, 13.211, 18.958,
                      24.611, 30.423, 36.076, 41.888, 47.541]
    x = [1, 2, 3, 4, 5, 6, 7, 8]

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
    y_without_heap = y_tlsf - heap_time

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Senden einer Übertragung")
    plt.plot(x, y, 'g')
    plt.plot(x, y_enqueue, 'g--')
    plt.plot(x, y_tlsf, 'r')
    plt.plot(x, y_tlsf_enqueue, 'r--')
    plt.plot(x, y_without_heap, 'y')

    plt.legend(["gesamt", "enqeueTransfer()",
               "gesamt mit tlsf", "tlsf enqeueTransfer()", "gesamt ohne heap"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y[i] - y[i - 1]
                                           for i in range(len(y) - 1, 1, -1)))) / (len(y) - 2)
    print("arduino send statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def rust_send_transfer(plt):
    y = [0, 3, 5, 7, 8, 10, 11, 12]
    x = [1, 2, 3, 4, 5, 6, 7, 8]

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Senden einer Übertragung")
    plt.plot(x, y, 'g')

    plt.legend(["gesamt"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y[i] - y[i - 1]
                                           for i in range(len(y) - 1, 1, -1)))) / (len(y) - 2)
    print("rust send statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def rust_rec_transfer(plt):
    y = [6, 15, 22, 22, 22, 22, 22, 44]
    y_no_realloc = [5, 9, 12, 12, 12, 12, 12, 29]
    y_subscr_func_reimpl_worst = [5, 9, 12, 15, 17, 20, 23, 26.5]
    y_subscr_func_reimpl_best = [2, 5, 8, 11, 14, 17, 20, 22.5]
    x = [1, 2, 3, 4, 5, 6, 7, 8]

    # plt.subplot(1, 2, 1)
    plt.title.set_text("Empfangen von Rahmen")
    plt.plot(x, y, 'g')
    plt.plot(x, y_no_realloc, 'r')
    plt.plot(x, y_subscr_func_reimpl_worst, 'b--')
    plt.plot(x, y_subscr_func_reimpl_best, 'b')

    plt.legend(["erste Messung", "erste Optimierung",
               "zweite Optimierung Worst Case", "zweite Optimierung Best Case"])

    plt.set_xlabel("Rahmen-Anzahl")

    elevation_from_two = sum(np.array(list(y[i] - y[i - 1]
                                           for i in range(len(y) - 1, 1, -1)))) / (len(y) - 2)
    print("rust receive statistics:")
    print("elev. from 2-8 frames: " + str(elevation_from_two) + " micros/frame")


def main():
    # f, (ax, bx) = plt.subplots(1, 2, sharey='row')
    # ax.set_ylabel("Mikrosekunden")
    # arduino_send_transfer(ax)
    # arduino_rec_frames(bx)
    f, (ax, bx) = plt.subplots(1, 2, sharey='row')
    ax.set_ylabel("Mikrosekunden")
    rust_send_transfer(ax)
    rust_rec_transfer(bx)

    plt.show()


if __name__ == "__main__":
    main()
