//! Explicit MSVC ABI profiles. Namespace decoration also changes type back-references;
//! adding a namespace substring to arbitrary symbols is not a valid mangling strategy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Namespace {
    Global,
    Qt,
    Isl0,
}

impl Namespace {
    pub(super) fn detect(
        major: u8,
        mut present: impl FnMut(&'static [u8]) -> bool,
    ) -> Result<Self, ()> {
        if present(Self::Global.symbol(QSTRING_UTF16_SYMBOL)?) {
            return Ok(Self::Global);
        }
        if cfg!(target_arch = "x86_64")
            && major == 6
            && present(Self::Qt.symbol(QSTRING_UTF16_SYMBOL)?)
        {
            return Ok(Self::Qt);
        }
        if cfg!(target_arch = "x86_64")
            && major == 6
            && present(Self::Isl0.symbol(QSTRING_UTF16_SYMBOL)?)
        {
            return Ok(Self::Isl0);
        }
        Err(())
    }
    pub(super) fn symbol(self, symbol: &'static [u8]) -> Result<&'static [u8], ()> {
        if self == Self::Global {
            #[cfg(target_arch = "x86")]
            return Ok(match symbol {
                DRAW_POINT_SIMPLE_SYMBOL => b"?drawText@QPainter@@QAEXABVQPointF@@ABVQString@@@Z\0",
                DRAW_POINT_SYMBOL => b"?drawText@QPainter@@QAEXABVQPointF@@ABVQString@@HH@Z\0",
                DRAW_RECT_SYMBOL => b"?drawText@QPainter@@QAEXABVQRect@@HABVQString@@PAV2@@Z\0",
                DRAW_RECT_OPTION_SYMBOL => {
                    b"?drawText@QPainter@@QAEXABVQRectF@@ABVQString@@ABVQTextOption@@@Z\0"
                }
                DRAW_RECT_F_SYMBOL => b"?drawText@QPainter@@QAEXABVQRectF@@HABVQString@@PAV2@@Z\0",
                DRAW_RECT_COORDS_SYMBOL => {
                    b"?drawText@QPainter@@QAEXHHHHHABVQString@@PAVQRect@@@Z\0"
                }
                QSTRING_UTF16_SYMBOL => b"?utf16@QString@@QBEPBGXZ\0",
                QSTRING_DTOR_SYMBOL => b"??1QString@@QAE@XZ\0",
                QSTRING5_CTOR_SYMBOL => b"??0QString@@QAE@PBVQChar@@H@Z\0",
                QSTRING5_SIZE_SYMBOL => b"?size@QString@@QBEHXZ\0",
                QAPPLICATION_ALL_WIDGETS_SYMBOL => {
                    b"?allWidgets@QApplication@@SA?AV?$QList@PAVQWidget@@@@XZ\0"
                }
                QWIDGET_FIND_SYMBOL => b"?find@QWidget@@SAPAV1@I@Z\0",
                QWIDGET_REPAINT_SYMBOL => b"?repaint@QWidget@@QAEXXZ\0",
                QLIST_DATA_DISPOSE_SYMBOL => b"?dispose@QListData@@SAXPAUData@1@@Z\0",
                _ => return Err(()),
            });
            #[cfg(not(target_arch = "x86"))]
            return Ok(symbol);
        }
        Ok(match self {
            Self::Qt => match symbol {
                DRAW_POINT_SIMPLE_SYMBOL => {
                    b"?drawText@QPainter@QT@@QEAAXAEBVQPointF@2@AEBVQString@2@@Z\0"
                }
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
                DRAW_RECT_COORDS_SYMBOL => {
                    b"?drawText@QPainter@QT@@QEAAXHHHHHAEBVQString@2@PEAVQRect@2@@Z\0"
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
            },
            Self::Isl0 => match symbol {
                DRAW_POINT_SIMPLE_SYMBOL => {
                    b"?drawText@QPainter@isl0@@QEAAXAEBVQPointF@2@AEBVQString@2@@Z\0"
                }
                DRAW_POINT_SYMBOL => b"?drawText@QPainter@isl0@@QEAAXAEBVQPointF@2@AEBVQString@2@HH@Z\0",
                DRAW_RECT_SYMBOL => {
                    b"?drawText@QPainter@isl0@@QEAAXAEBVQRect@2@HAEBVQString@2@PEAV32@@Z\0"
                }
                DRAW_RECT_OPTION_SYMBOL => {
                    b"?drawText@QPainter@isl0@@QEAAXAEBVQRectF@2@AEBVQString@2@AEBVQTextOption@2@@Z\0"
                }
                DRAW_RECT_F_SYMBOL => {
                    b"?drawText@QPainter@isl0@@QEAAXAEBVQRectF@2@HAEBVQString@2@PEAV32@@Z\0"
                }
                DRAW_RECT_COORDS_SYMBOL => {
                    b"?drawText@QPainter@isl0@@QEAAXHHHHHAEBVQString@2@PEAVQRect@2@@Z\0"
                }
                QSTRING_UTF16_SYMBOL => b"?utf16@QString@isl0@@QEBAPEBGXZ\0",
                QSTRING_DTOR_SYMBOL => b"??1QString@isl0@@QEAA@XZ\0",
                QSTRING6_CTOR_SYMBOL => b"??0QString@isl0@@QEAA@PEBVQChar@1@_J@Z\0",
                QSTRING6_SIZE_SYMBOL => b"?size@QString@isl0@@QEBA_JXZ\0",
                QAPPLICATION_ALL_WIDGETS_SYMBOL => {
                    b"?allWidgets@QApplication@isl0@@SA?AV?$QList@PEAVQWidget@isl0@@@2@XZ\0"
                }
                QWIDGET_FIND_SYMBOL => b"?find@QWidget@isl0@@SAPEAV12@_K@Z\0",
                QWIDGET_REPAINT_SYMBOL => b"?repaint@QWidget@isl0@@QEAAXXZ\0",
                QARRAY_DATA_DEALLOCATE_SYMBOL => {
                    b"?deallocate@QArrayData@isl0@@SAXPEAU12@_J1@Z\0"
                }
                _ => return Err(()),
            },
            Self::Global => unreachable!(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(target_arch = "x86_64")]
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
    #[cfg(target_arch = "x86_64")]
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
    #[cfg(target_arch = "x86_64")]
    fn silhouette_isl0_qt6_namespace_is_recognized() {
        let utf16 = b"?utf16@QString@isl0@@QEBAPEBGXZ\0";
        let namespace = Namespace::detect(6, |symbol| symbol == utf16).unwrap();
        assert_eq!(namespace, Namespace::Isl0);
        assert_eq!(
            namespace.symbol(DRAW_RECT_SYMBOL).unwrap(),
            b"?drawText@QPainter@isl0@@QEAAXAEBVQRect@2@HAEBVQString@2@PEAV32@@Z\0"
        );
        assert_eq!(
            namespace.symbol(DRAW_POINT_SIMPLE_SYMBOL).unwrap(),
            b"?drawText@QPainter@isl0@@QEAAXAEBVQPointF@2@AEBVQString@2@@Z\0"
        );
        assert_eq!(
            namespace.symbol(DRAW_RECT_COORDS_SYMBOL).unwrap(),
            b"?drawText@QPainter@isl0@@QEAAXHHHHHAEBVQString@2@PEAVQRect@2@@Z\0"
        );
        assert_eq!(
            namespace.symbol(QAPPLICATION_ALL_WIDGETS_SYMBOL).unwrap(),
            b"?allWidgets@QApplication@isl0@@SA?AV?$QList@PEAVQWidget@isl0@@@2@XZ\0"
        );
    }

    #[test]
    fn unknown_profiles_and_unverified_qt5_namespace_fail_closed() {
        assert!(Namespace::detect(6, |_| false).is_err());
        let utf16 = Namespace::Global.symbol(QSTRING_UTF16_SYMBOL).unwrap();
        assert!(Namespace::detect(5, |symbol| symbol != utf16).is_err());
        assert!(Namespace::Qt.symbol(QSTRING5_CTOR_SYMBOL).is_err());
        assert!(Namespace::Qt.symbol(b"unknown\0").is_err());
    }
    #[test]
    #[cfg(target_arch = "x86")]
    fn qt5_x86_selects_member_and_static_decorations() {
        let utf16 = b"?utf16@QString@@QBEPBGXZ\0";
        assert_eq!(
            Namespace::detect(5, |symbol| symbol == utf16),
            Ok(Namespace::Global)
        );
        assert_eq!(
            Namespace::Global.symbol(QWIDGET_FIND_SYMBOL).unwrap(),
            b"?find@QWidget@@SAPAV1@I@Z\0"
        );
        assert!(Namespace::Global.symbol(QSTRING6_CTOR_SYMBOL).is_err());
    }
}
