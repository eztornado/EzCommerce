import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../constants/app_colors.dart';

/// Tema de la aplicación replicando el design system del shop Bagisto:
/// Poppins para todo + DM Serif Display para títulos de sección,
/// radios xl (12) / 2xl (16), sin dark mode.
abstract final class EzTheme {
  static const double radiusLg = 8;
  static const double radiusXl = 12;
  static const double radius2Xl = 16;

  static ThemeData light() {
    const base = ColorScheme.light(
      primary: AppColors.navyBlue,
      onPrimary: Colors.white,
      secondary: AppColors.navyBlue,
      surface: Colors.white,
      error: AppColors.red600,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: base,
      scaffoldBackgroundColor: Colors.white,
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.white,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        systemOverlayStyle: SystemUiOverlayStyle.dark,
        iconTheme: IconThemeData(color: AppColors.gray900),
        titleTextStyle: TextStyle(
          fontFamily: 'Poppins',
          fontSize: 24,
          fontWeight: FontWeight.w500,
          color: AppColors.gray900,
        ),
      ),
      textTheme: const TextTheme(
        displayLarge: TextStyle(fontFamily: 'Poppins', fontSize: 39, fontWeight: FontWeight.w500, color: AppColors.gray900),
        displayMedium: TextStyle(fontFamily: 'DM Serif Display', fontSize: 30, color: AppColors.navyBlue),
        headlineMedium: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500, color: AppColors.gray900),
        headlineSmall: TextStyle(fontFamily: 'Poppins', fontSize: 20, fontWeight: FontWeight.w500, color: AppColors.gray900),
        titleLarge: TextStyle(fontFamily: 'Poppins', fontSize: 18, fontWeight: FontWeight.w500, color: AppColors.gray900),
        titleMedium: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500, color: AppColors.gray900),
        titleSmall: TextStyle(fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500, color: AppColors.gray900),
        bodyLarge: TextStyle(fontFamily: 'Poppins', fontSize: 16, color: AppColors.gray900),
        bodyMedium: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.gray900),
        bodySmall: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500),
        labelLarge: TextStyle(fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500, color: AppColors.gray900),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: AppColors.zinc100,
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusXl),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusXl),
          borderSide: const BorderSide(color: Colors.transparent),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusXl),
          borderSide: const BorderSide(color: AppColors.gray400),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(radiusXl),
          borderSide: const BorderSide(color: AppColors.red600),
        ),
        hintStyle: const TextStyle(fontFamily: 'Poppins', fontSize: 12, fontWeight: FontWeight.w500, color: AppColors.gray400),
      ),
      dropdownMenuTheme: DropdownMenuThemeData(
        inputDecorationTheme: InputDecorationTheme(
          filled: true,
          fillColor: Colors.white,
          contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(radiusXl),
            borderSide: const BorderSide(color: AppColors.zinc200),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(radiusXl),
            borderSide: const BorderSide(color: AppColors.zinc200),
          ),
        ),
      ),
      dividerTheme: const DividerThemeData(color: AppColors.zinc200, thickness: 1),
      snackBarTheme: const SnackBarThemeData(
        backgroundColor: AppColors.navyBlue,
        contentTextStyle: TextStyle(fontFamily: 'Poppins', color: Colors.white),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}

/// Botón primario del shop: navy, radius 2xl, padding generoso.
class PrimaryButton extends StatelessWidget {
  const PrimaryButton({
    super.key,
    required this.label,
    this.onPressed,
    this.expand = false,
    this.isLoading = false,
  });

  final String label;
  final VoidCallback? onPressed;
  final bool expand;
  final bool isLoading;

  @override
  Widget build(BuildContext context) {
    final child = isLoading
        ? const SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
          )
        : Text(
            label,
            style: const TextStyle(
              fontFamily: 'Poppins',
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: Colors.white,
            ),
          );
    return SizedBox(
      width: expand ? double.infinity : null,
      child: TextButton(
        onPressed: isLoading ? null : onPressed,
        style: TextButton.styleFrom(
          backgroundColor: AppColors.navyBlue,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 44, vertical: 14),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radius2Xl)),
        ),
        child: child,
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
  });

  final String label;
  final VoidCallback? onPressed;
  final bool expand;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: expand ? double.infinity : null,
      child: TextButton(
        onPressed: onPressed,
        style: TextButton.styleFrom(
          backgroundColor: Colors.white,
          foregroundColor: AppColors.navyBlue,
          side: const BorderSide(color: AppColors.navyBlue, width: 2),
          padding: const EdgeInsets.symmetric(horizontal: 44, vertical: 12),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radius2Xl)),
        ),
        child: Text(
          label,
          style: const TextStyle(
            fontFamily: 'Poppins',
            fontSize: 14,
            fontWeight: FontWeight.w500,
            color: AppColors.navyBlue,
          ),
        ),
      ),
    );
  }
}
