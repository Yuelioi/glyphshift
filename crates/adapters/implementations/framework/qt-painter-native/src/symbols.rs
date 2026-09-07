//! Explicit MSVC ABI profiles. Namespace decoration also changes type back-references;
//! adding a namespace substring to arbitrary symbols is not a valid mangling strategy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Namespace {
    Global,
    Qt,
}

impl Namespace {
    pub(super) fn detect(
        major: u8,
        mut present: impl FnMut(&'static [u8]) -> bool,
    ) -> Result<Self, ()> {
        if present(QSTRING_UTF16_SYMBOL) {
            return Ok(Self::Global);
        }
        if major == 6 && present(Self::Qt.symbol(QSTRING_UTF16_SYMBOL)?) {
            return Ok(Self::Qt);
        }
        Err(())
    }
    pub(super) fn symbol(self, symbol: &'static [u8]) -> Result<&'static [u8], ()> {
        if self == Self::Global {
            return Ok(symbol);
        }
        Ok(match symbol {
            DRAW_POINT_SYMBOL => b"?drawText@QPainter@QT@@QEAAXAEBVQPointF@2@AEBVQString@2@HH@Z\0",
            DRAW_RECT_SYMBOL => {
                b"?drawText@QPainter@QT@@QEAAXAEBVQRect@2@HAEBVQString@2@PEAV32@@Z\0"
            }
            DRAW_RECT_OPTION_SYMBOL => {
                b"?drawText@QPainter@QT@@QEAAXAEBVQRectF@2@AEBVQString@2@AEBVQTextOption@2@@Z\0"
            }
            DRAW_RECT_F_SYMBOL => {
                b"?drawText@QPainter@QT@@QEAAXAEBVQRectF@2@HAEBVQString@2@PEAV32@@Z\0"
            }
            QSTRING_UTF16_SYMBOL => b"?utf16@QString@QT@@QEBAPEBGXZ\0",
            QSTRING_DTOR_SYMBOL => b"??1QString@QT@@QEAA@XZ\0",
            QSTRING6_CTOR_SYMBOL => b"??0QString@QT@@QEAA@PEBVQChar@1@_J@Z\0",
            QSTRING6_SIZE_SYMBOL => b"?size@QString@QT@@QEBA_JXZ\0",
            QAPPLICATION_ALL_WIDGETS_SYMBOL => {
                b"?allWidgets@QApplication@QT@@SA?AV?$QList@PEAVQWidget@QT@@@2@XZ\0"
            }
            QWIDGET_FIND_SYMBOL => b"?find@QWidget@QT@@SAPEAV12@_K@Z\0",
            QWIDGET_REPAINT_SYMBOL => b"?repaint@QWidget@QT@@QEAAXXZ\0",
            QARRAY_DATA_DEALLOCATE_SYMBOL => b"?deallocate@QArrayData@QT@@SAXPEAU12@_J1@Z\0",
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn namespaced_core_selects_a_consistent_profile() {
        let utf16 = b"?utf16@QString@QT@@QEBAPEBGXZ\0";
        let namespace = Namespace::detect(6, |symbol| symbol == utf16).unwrap();
        assert_eq!(namespace, Namespace::Qt);
        assert_eq!(
            namespace.symbol(DRAW_RECT_SYMBOL).unwrap(),
            b"?drawText@QPainter@QT@@QEAAXAEBVQRect@2@HAEBVQString@2@PEAV32@@Z\0"
        );
        assert_ne!(
            namespace.symbol(QWIDGET_REPAINT_SYMBOL).unwrap(),
            QWIDGET_REPAINT_SYMBOL
        );
    }
    #[test]
    fn standard_qt_preserves_existing_symbols() {
        for major in [5, 6] {
            let namespace =
                Namespace::detect(major, |symbol| symbol == QSTRING_UTF16_SYMBOL).unwrap();
            assert_eq!(
                namespace.symbol(DRAW_RECT_SYMBOL).unwrap(),
                DRAW_RECT_SYMBOL
            );
            assert_eq!(
                namespace.symbol(QSTRING5_CTOR_SYMBOL).unwrap(),
                QSTRING5_CTOR_SYMBOL
            );
        }
    }
    #[test]
    fn unknown_profiles_and_unverified_qt5_namespace_fail_closed() {
        assert!(Namespace::detect(6, |_| false).is_err());
        assert!(Namespace::detect(5, |symbol| symbol != QSTRING_UTF16_SYMBOL).is_err());
        assert!(Namespace::Qt.symbol(QSTRING5_CTOR_SYMBOL).is_err());
        assert!(Namespace::Qt.symbol(b"unknown\0").is_err());
    }
}
