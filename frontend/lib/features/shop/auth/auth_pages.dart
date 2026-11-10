import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Molde de auth: logo centrado + tarjeta max-width + copyright.
class AuthCard extends StatelessWidget {
  const AuthCard({super.key, required this.title, required this.subtitle, required this.child});
  final String title;
  final String subtitle;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Column(
              children: [
                GestureDetector(
                  onTap: () => context.go('/'),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: const [
                      Icon(Icons.storefront, color: AppColors.navyBlue, size: 30),
                      SizedBox(width: 8),
                      Text('EzCommerce',
                          style: TextStyle(
                              fontFamily: 'DM Serif Display', fontSize: 26, color: AppColors.navyBlue)),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
                Container(
                  padding: const EdgeInsets.all(48),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                    border: Border.all(color: AppColors.zinc200),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(title,
                          style: const TextStyle(
                              fontFamily: 'DM Serif Display', fontSize: 32, color: AppColors.gray900)),
                      const SizedBox(height: 4),
                      Text(subtitle,
                          style: TextStyle(fontFamily: 'Poppins', fontSize: 16, color: AppColors.zinc500)),
                      const SizedBox(height: 24),
                      child,
                    ],
                  ),
                ),
                const SizedBox(height: 16),
                Text(
                  '© Copyright ${DateTime.now().year}, EzCommerce.',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class SignInPage extends StatefulWidget {
  const SignInPage({super.key});

  @override
  State<SignInPage> createState() => _SignInPageState();
}

class _SignInPageState extends State<SignInPage> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  bool _obscure = true;

  @override
  Widget build(BuildContext context) {
    final auth = context.watch<AuthProvider>();
    return AuthCard(
      title: 'Sign In',
      subtitle: 'Please enter your login details below.',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (auth.error != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Text(auth.error!, style: const TextStyle(color: AppColors.red600, fontSize: 13)),
            ),
          const Text('Email', style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          TextField(controller: _email, keyboardType: TextInputType.emailAddress),
          const SizedBox(height: 16),
          const Text('Password', style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          TextField(
            controller: _password,
            obscureText: _obscure,
            decoration: InputDecoration(
              suffixIcon: IconButton(
                icon: Icon(_obscure ? Icons.visibility_outlined : Icons.visibility_off_outlined, size: 18),
                onPressed: () => setState(() => _obscure = !_obscure),
              ),
            ),
          ),
          Align(
            alignment: Alignment.centerRight,
            child: TextButton(
              onPressed: () => context.go('/forgot-password'),
              child: const Text('Forgot Password?',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.navyBlue)),
            ),
          ),
          const SizedBox(height: 8),
          PrimaryButton(
            label: 'Sign In',
            expand: true,
            isLoading: auth.loading,
            onPressed: () async {
              final ok = await auth.login(_email.text, _password.text);
              if (ok && context.mounted) context.go('/account');
            },
          ),
          const SizedBox(height: 16),
          Center(
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Text('New Customer?',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                TextButton(
                  onPressed: () => context.go('/register'),
                  child: const Text('Create Your Account',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.navyBlue)),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class SignUpPage extends StatefulWidget {
  const SignUpPage({super.key});

  @override
  State<SignUpPage> createState() => _SignUpPageState();
}

class _SignUpPageState extends State<SignUpPage> {
  final _formKey = GlobalKey<FormState>();
  final _firstName = TextEditingController();
  final _lastName = TextEditingController();
  final _email = TextEditingController();
  final _password = TextEditingController();
  final _confirmation = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final auth = context.watch<AuthProvider>();
    return AuthCard(
      title: 'Sign Up',
      subtitle: 'Create your account in seconds.',
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (auth.error != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Text(auth.error!, style: const TextStyle(color: AppColors.red600, fontSize: 13)),
              ),
            Row(children: [
              Expanded(child: _field('First name', _firstName)),
              const SizedBox(width: 12),
              Expanded(child: _field('Last name', _lastName)),
            ]),
            const SizedBox(height: 12),
            _field('Email', _email, email: true),
            const SizedBox(height: 12),
            _field('Password', _password, obscure: true),
            const SizedBox(height: 12),
            _field('Confirm Password', _confirmation, obscure: true),
            const SizedBox(height: 20),
            PrimaryButton(
              label: 'Register',
              expand: true,
              isLoading: auth.loading,
              onPressed: () async {
                if (!_formKey.currentState!.validate()) return;
                final ok = await auth.register(
                  firstName: _firstName.text,
                  lastName: _lastName.text,
                  email: _email.text,
                  password: _password.text,
                  passwordConfirmation: _confirmation.text,
                );
                if (ok && context.mounted) context.go('/account');
              },
            ),
            const SizedBox(height: 16),
            Center(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('Already have an account?',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  TextButton(
                    onPressed: () => context.go('/login'),
                    child: const Text('Sign In',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.navyBlue)),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _field(String label, TextEditingController controller,
      {bool obscure = false, bool email = false}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
        TextFormField(
          controller: controller,
          obscureText: obscure,
          keyboardType: email ? TextInputType.emailAddress : null,
          decoration: const InputDecoration(fillColor: Colors.white, border: OutlineInputBorder()),
          validator: (value) => (value ?? '').trim().isEmpty ? 'This field is required.' : null,
        ),
      ],
    );
  }
}

class ForgotPasswordPage extends StatefulWidget {
  const ForgotPasswordPage({super.key});

  @override
  State<ForgotPasswordPage> createState() => _ForgotPasswordPageState();
}

class _ForgotPasswordPageState extends State<ForgotPasswordPage> {
  final _email = TextEditingController();
  String? _message;

  @override
  Widget build(BuildContext context) {
    return AuthCard(
      title: 'Forgot Password',
      subtitle: 'Enter your email to receive a reset link.',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(controller: _email, keyboardType: TextInputType.emailAddress),
          const SizedBox(height: 16),
          PrimaryButton(
            label: 'Reset Password',
            expand: true,
            onPressed: () async {
              try {
                await apiPost('customer/forgot-password', body: {'email': _email.text});
                if (mounted) {
                  setState(() => _message = 'If your account exists, you will receive an email shortly.');
                }
              } catch (error) {
                if (mounted) setState(() => _message = '$error');
              }
            },
          ),
          if (_message != null)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(_message!,
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc600)),
            ),
        ],
      ),
    );
  }
}
