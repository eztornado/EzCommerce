import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../constants/app_colors.dart';

/// Design system del shop, alineado con Bagisto:
/// Poppins para UI + DM Serif Display para titulares, radios 8/12/16.
abstract final class EzTheme {
  static const double radiusLg = 8;
  static const double radiusXl = 12;
  static const double radius2Xl = 16;

  static const String sans = 'Poppins';
  static const String serif = 'DM Serif Display';

  static ThemeData light() {
    const base = ColorScheme.light(
      primary: AppColors.navyBlue,
      onPrimary: Colors.white,
      secondary: AppColors.navyBlue,
      onSecondary: Colors.white,
      surface: Colors.white,
      onSurface: AppColors.gray900,
      error: AppColors.red600,
    );

    final textTheme = const TextTheme(
      displayLarge: TextStyle(
          fontFamily: serif, fontSize: 56, height: 1.05, color: AppColors.navyBlue),
      displayMedium: TextStyle(
          fontFamily: serif, fontSize: 40, height: 1.1, color: AppColors.navyBlue),
      displaySmall: TextStyle(
          fontFamily: serif, fontSize: 32, height: 1.15, color: AppColors.gray900),
      headlineMedium: TextStyle(
          fontFamily: sans, fontSize: 24, fontWeight: FontWeight.w500, color: AppColors.gray900),
      headlineSmall: TextStyle(
          fontFamily: sans, fontSize: 20, fontWeight: FontWeight.w500, color: AppColors.gray900),
      titleLarge: TextStyle(
          fontFamily: sans, fontSize: 18, fontWeight: FontWeight.w500, color: AppColors.gray900),
      titleMedium: TextStyle(
          fontFamily: sans, fontSize: 16, fontWeight: FontWeight.w500, color: AppColors.gray900),
      titleSmall: TextStyle(
          fontFamily: sans, fontSize: 14, fontWeight: FontWeight.w500, color: AppColors.gray900),
      bodyLarge: TextStyle(fontFamily: sans, fontSize: 16, color: AppColors.gray900),
      bodyMedium: TextStyle(fontFamily: sans, fontSize: 14, color: AppColors.gray900),
      bodySmall: TextStyle(fontFamily: sans, fontSize: 12, color: AppColors.zinc500),
      labelLarge: TextStyle(
          fontFamily: sans, fontSize: 14, fontWeight: FontWeight.w500, color: AppColors.gray900),
      labelMedium: TextStyle(
          fontFamily: sans, fontSize: 12, fontWeight: FontWeight.w500, color: AppColors.gray900),
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: base,
      fontFamily: sans,
      textTheme: textTheme,
      scaffoldBackgroundColor: Colors.white,
      splashFactory: InkRipple.splashFactory,
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.white,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        scrolledUnderElevation: 0,
        systemOverlayStyle: SystemUiOverlayStyle.dark,
        iconTheme: IconThemeData(color: AppColors.gray900),
        titleTextStyle: TextStyle(
          fontFamily: sans,
          fontSize: 18,
          fontWeight: FontWeight.w500,
          color: AppColors.gray900,
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: AppColors.zinc100,
        hintStyle: const TextStyle(
            fontFamily: sans, fontSize: 13, fontWeight: FontWeight.w400, color: AppColors.zinc400),
        contentPadding: const EdgeInsets.symmetric(horizontal: 18, vertical: 16),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusLg),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusLg),
          borderSide: BorderSide.none,
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusLg),
          borderSide: const BorderSide(color: AppColors.zinc400),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusLg),
          borderSide: const BorderSide(color: AppColors.red600),
        ),
      ),
      dropdownMenuTheme: DropdownMenuThemeData(
        textStyle: const TextStyle(fontFamily: sans, fontSize: 14, color: AppColors.gray900),
        menuStyle: MenuStyle(
          backgroundColor: const WidgetStatePropertyAll(Colors.white),
          elevation: const WidgetStatePropertyAll(8),
          shape: WidgetStatePropertyAll(RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radiusLg),
            side: const BorderSide(color: AppColors.zinc200),
          )),
        ),
      ),
      menuTheme: MenuThemeData(
        style: MenuStyle(
          backgroundColor: const WidgetStatePropertyAll(Colors.white),
          elevation: const WidgetStatePropertyAll(12),
          surfaceTintColor: const WidgetStatePropertyAll(Colors.white),
          shape: WidgetStatePropertyAll(RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radiusLg),
            side: const BorderSide(color: AppColors.zinc200),
          )),
        ),
      ),
      dividerTheme: const DividerThemeData(color: AppColors.zinc200, thickness: 1, space: 1),
      snackBarTheme: const SnackBarThemeData(
        backgroundColor: AppColors.navyBlue,
        contentTextStyle: TextStyle(fontFamily: sans, color: Colors.white),
        behavior: SnackBarBehavior.floating,
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: AppColors.navyBlue,
          textStyle: const TextStyle(fontFamily: sans, fontWeight: FontWeight.w500),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(radiusLg)),
        ),
      ),
      iconButtonTheme: IconButtonThemeData(
        style: IconButton.styleFrom(foregroundColor: AppColors.gray900),
      ),
      tooltipTheme: TooltipThemeData(
        decoration: BoxDecoration(
          color: AppColors.navyBlue,
          borderRadius: BorderRadius.circular(6),
        ),
        textStyle: const TextStyle(fontFamily: sans, fontSize: 12, color: Colors.white),
      ),
    );
  }
}

/// Botón primario del shop: navy, radius 12, 48 px de alto.
class PrimaryButton extends StatelessWidget {
  const PrimaryButton({
    super.key,
    required this.label,
    this.onPressed,
    this.expand = false,
    this.isLoading = false,
    this.icon,
    this.height = 48,
  });

  final String label;
  final VoidCallback? onPressed;
  final bool expand;
  final bool isLoading;
  final IconData? icon;
  final double height;

  @override
  Widget build(BuildContext context) {
    final disabled = onPressed == null || isLoading;
    return SizedBox(
      width: expand ? double.infinity : null,
      height: height,
      child: MouseRegion(
        cursor: disabled ? SystemMouseCursors.basic : SystemMouseCursors.click,
        child: TextButton(
          onPressed: disabled ? null : onPressed,
          style: TextButton.styleFrom(
            backgroundColor: AppColors.navyBlue,
            foregroundColor: Colors.white,
            disabledBackgroundColor: AppColors.zinc300,
            disabledForegroundColor: Colors.white,
            padding: const EdgeInsets.symmetric(horizontal: 28),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radiusXl)),
            overlayColor: Colors.white.withValues(alpha: 0.08),
          ),
          child: isLoading
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                )
              : Row(
                  mainAxisSize: MainAxisSize.min,
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    if (icon != null) ...[
                      Icon(icon, size: 18, color: Colors.white),
                      const SizedBox(width: 8),
                    ],
                    Text(
                      label,
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 14,
                        fontWeight: FontWeight.w500,
                        color: Colors.white,
                      ),
                    ),
                  ],
                ),
        ),
      ),
    );
  }
}

/// Botón secundario: borde navy, hover navy al 5%.
class SecondaryButton extends StatelessWidget {
  const SecondaryButton({
    super.key,
    required this.label,
    this.onPressed,
    this.expand = false,
    this.icon,
    this.height = 48,
  });

  final String label;
  final VoidCallback? onPressed;
  final bool expand;
  final IconData? icon;
  final double height;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: expand ? double.infinity : null,
      height: height,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: TextButton(
          onPressed: onPressed,
          style: TextButton.styleFrom(
            backgroundColor: Colors.white,
            foregroundColor: AppColors.navyBlue,
            side: const BorderSide(color: AppColors.navyBlue, width: 1.5),
            padding: const EdgeInsets.symmetric(horizontal: 26),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radiusXl)),
            overlayColor: AppColors.navyBlue.withValues(alpha: 0.05),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (icon != null) ...[
                Icon(icon, size: 18, color: AppColors.navyBlue),
                const SizedBox(width: 8),
              ],
              Text(
                label,
                style: const TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                  color: AppColors.navyBlue,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
