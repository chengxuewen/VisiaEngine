// VisiaEngineWidget -- header-only Qt6 host (zero Q_OBJECT = zero moc; Qt round batch v1.3/Q2).
// Four contract nails: (1) the GUI thread = the engine owner (CAPI-03, zero marshaling); (2) attach in showEvent
// (winId already realized; repeated show = re-attach, CAPI-06); (3) WA_PaintOnScreen + empty paint --
// Qt never paints over the engine canvas; (4) the frame pump QTimer(60Hz) -> visiaengine_render (D8 trigger (1) proof face).
#pragma once

#include <QtGlobal>
#include <QMouseEvent>
#include <QTimer>
#include <QWheelEvent>
#include <QWidget>

#include <X11/Xlib.h>  // conda qt6-main does not package QX11Application (the missing x11extras is proven) --
                       // the host holds its own Display* (an xid across connections is legal: attach demo_x11 same-machine evidence;
                       // the only X calls are XOpenDisplay/XSync, zero contact with the PIT-16 self-kill surface)

#include "visiaengine.h"

class VisiaEngineWidget : public QWidget {
public:
    explicit VisiaEngineWidget(uint64_t ve, QWidget* parent = nullptr) : QWidget(parent), ve_(ve) {
        setAttribute(Qt::WA_NativeWindow);
        setAttribute(Qt::WA_PaintOnScreen);      // the engine draws directly, Qt backingstore bypassed
        setAttribute(Qt::WA_NoSystemBackground); // no white flash on the first frame
        setFocusPolicy(Qt::StrongFocus);
        // function-pointer style connect: no moc dependency
        QObject::connect(&pump_, &QTimer::timeout, this, [this] { tick(); });
    }

    /// max_frames <= 0 = interactive infinite; N > 0 quits after N frames (smoke face)
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
            dpy_ = XOpenDisplay(nullptr);  // a parallel connection with Qt(xcb): the xid shares the protocol
            if (!dpy_) {
                qWarning("XOpenDisplay failed (no X? do not run under wayland [Qt round §1-i])");
                return;
            }
        }
        Window xid = static_cast<Window>(winId());
        XSync(dpy_, False);  // let the server acknowledge the window
        const int rc = visiaengine_attach(ve_, static_cast<uint64_t>(xid),
                                          reinterpret_cast<uint64_t>(dpy_),
                                          0 /* kind: x11 */);
        if (rc != VE_OK) {
            qWarning("attach: %s", visiaengine_last_error(ve_));
        }
    }

    void paintEvent(QPaintEvent*) override { /* the engine self-draws, Qt idles */ }

    void mousePressEvent(QMouseEvent* e) override { send(2, e->position().x(), e->position().y(), 0, e->button()); }
    void mouseReleaseEvent(QMouseEvent* e) override { send(3, e->position().x(), e->position().y(), 0, e->button()); }
    void mouseMoveEvent(QMouseEvent* e) override { send(1, e->position().x(), e->position().y(), 0, Qt::NoButton); }
    void wheelEvent(QWheelEvent* e) override { send(4, e->position().x(), e->position().y(), e->angleDelta().y() / 120.0f, Qt::NoButton); }

    void resizeEvent(QResizeEvent* e) override {
        (void)visiaengine_viewport(ve_, static_cast<uint32_t>(e->size().width()),
                                   static_cast<uint32_t>(e->size().height())); // the Q1 behavior-lock contract face
    }

private:
    void send(int kind, qreal x, qreal y, float wheel, Qt::MouseButton button) {
        VeInput in_{};
        in_.struct_size = sizeof(VeInput);
        in_.kind = static_cast<uint32_t>(kind); // 1=move 2=down 3=up 4=wheel (engine.rs as-recorded)
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
            qApp->quit(); // smoke exit: console assertions are printed by the host process
        }
    }

    uint64_t ve_ = 0;
    Display* dpy_ = nullptr;  // the host holds its own X connection (the attach contract requires display, CAPI-06)
    QTimer pump_;
    int max_frames_ = 0;
    int frames_ = 0;
};
