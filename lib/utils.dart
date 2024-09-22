import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class Utils {
  static TextStyle getCustomFont(
      {required String fontFamily,
      required double fontSize,
      FontWeight fontWeight = FontWeight.normal,
      Color color = Colors.black,
      TextDecoration decoration = TextDecoration.none}) {
    return GoogleFonts.getFont(
      fontFamily,
      fontSize: fontSize,
      fontWeight: fontWeight,
      color: color,
      decoration: decoration,
    );
  }
}
