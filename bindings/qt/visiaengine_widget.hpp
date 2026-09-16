// VisiaEngineWidget —— header-only Qt6 宿主（零 Q_OBJECT=零 moc；批 Qt 轮 v1.3/Q2）。
// 合同四钉：① GUI 线程=引擎 owner（CAPI-03 零 marshal）；② attach 于 showEvent
//（winId 已 realized；重复 show=重 attach，CAPI-06）；③ WA_PaintOnScreen+空 paint——
// Qt 永不向引擎画布刷皮；④ 帧泵 QTimer(60Hz)→visiaengine_render（D8 触发器① 实证面）。
#pragma once

#include <QtGlobal>
#include <QMouseEvent>
#include <QTimer>
#include <QWheelEvent>
#include <QWidget>

#include <X11/Xlib.h>  // conda qt6-main 未打包 QX11Application（x11extras 缺位实锤）——
                       // Display* 由宿主自持（xid 跨连接合法：attach demo_x11 同机件实证；
                       // 全 XCall 仅 XOpenDisplay/XSync，PIT-16 自杀面零触达）

#include "visiaengine.h"

class VisiaEngineWidget : public QWidget {
public:
    explicit VisiaEngineWidget(uint64_t ve, QWidget* parent = nullptr) : QWidget(parent), ve_(ve) {
        setAttribute(Qt::WA_NativeWindow);
        setAttribute(Qt::WA_PaintOnScreen);      // 引擎直绘，Qt  backingstore 旁路
        setAttribute(Qt::WA_NoSystemBackground); // 首帧不刷白
        setFocusPolicy(Qt::StrongFocus);
        // 函数指针式 connect：无 moc 依赖
        QObject::connect(&pump_, &QTimer::timeout, this, [this] { tick(); });
    }

    /// max_frames<=0=交互无限；N>0 跑 N 帧后 quit（smoke 面）
    void start(int max_frames) {
        max_frames_ = max_frames;
        pump_.start();
    }

    uint64_t engine() const { return ve_; }

    ~VisiaEngineWidget() override {
        if (dpy_) {
            XCloseDisplay(dpy_);
        }
    }

protected:
    void showEvent(QShowEvent* ev) override {
        QWidget::showEvent(ev);
        if (!dpy_) {
            dpy_ = XOpenDisplay(nullptr);  // 与 Qt(xcb) 并行连接：xid 共享协议
            if (!dpy_) {
                qWarning("XOpenDisplay 失败（无 X？wayland 不装 [Qt 轮§1①]）");
                return;
            }
        }
        Window xid = static_cast<Window>(winId());
        XSync(dpy_, False);  // 让服务端认账窗口
        const int rc = visiaengine_attach(ve_, static_cast<uint64_t>(xid),
                                          reinterpret_cast<uint64_t>(dpy_),
                                          0 /* kind: x11 */);
        if (rc != VE_OK) {
            qWarning("attach: %s", visiaengine_last_error(ve_));
        }
    }

    void paintEvent(QPaintEvent*) override { /* 引擎自绘，Qt 空转 */ }

    void mousePressEvent(QMouseEvent* e) override { send(2, e->position().x(), e->position().y(), 0, e->button()); }
    void mouseReleaseEvent(QMouseEvent* e) override { send(3, e->position().x(), e->position().y(), 0, e->button()); }
    void mouseMoveEvent(QMouseEvent* e) override { send(1, e->position().x(), e->position().y(), 0, Qt::NoButton); }
    void wheelEvent(QWheelEvent* e) override { send(4, e->position().x(), e->position().y(), e->angleDelta().y() / 120.0f, Qt::NoButton); }

    void resizeEvent(QResizeEvent* e) override {
        (void)visiaengine_viewport(ve_, static_cast<uint32_t>(e->size().width()),
                                   static_cast<uint32_t>(e->size().height())); // Q1 行为锁合同面
    }

private:
    void send(int kind, qreal x, qreal y, float wheel, Qt::MouseButton button) {
        VeInput in_{};
        in_.struct_size = sizeof(VeInput);
        in_.kind = static_cast<uint32_t>(kind); // 1=move 2=down 3=up 4=wheel（engine.rs 实录）
        in_.px = static_cast<float>(x);
        in_.py = static_cast<float>(y);
        in_.wheel = wheel;
        in_.button = static_cast<uint32_t>(button);
        (void)visiaengine_on_input(ve_, &in_);
    }

    void tick() {
        if (visiaengine_render(ve_) != VE_OK) {
            pump_.stop();
            return;
        }
        if (max_frames_ > 0 && ++frames_ >= max_frames_) {
            pump_.stop();
            qApp->quit(); // smoke 出口：控制台断言由宿主进程打
        }
    }

    uint64_t ve_ = 0;
    Display* dpy_ = nullptr;  // 宿主自持 X 连接（attach 合同 display 必填，CAPI-06）
    QTimer pump_;
    int max_frames_ = 0;
    int frames_ = 0;
};
